mod buffer_make;
mod command_buffers;
mod framebuffer;
mod init_renderer_state;
mod instance_create;
mod lib_select;
mod make_swapchain;
mod pipeline;
mod queue_device_creation;
mod render_pass;
mod select_physical_device;
mod subpass;
mod surface_create;
use std::{sync::Arc, time::Duration};
use tracing::{error, instrument};

use try_log::log_tries;
use vk::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vk::pipeline::graphics::input_assembly::InputAssemblyState;
use vk::pipeline::graphics::multisample::MultisampleState;
use vk::pipeline::graphics::rasterization::RasterizationState;
use vk::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vk::pipeline::graphics::viewport::{Viewport, ViewportState};
use vk::pipeline::graphics::GraphicsPipelineCreateInfo;
use vk::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vk::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vk::render_pass::{Framebuffer, Subpass};
use vk::swapchain::SwapchainPresentInfo;
use vk::{buffer::Subbuffer, sync::GpuFuture};
use vk::{Validated, VulkanError};
use vulkano as vk;
use winit::window::Window;
mod types;

mod consts;

mod renderer_error;
pub use renderer_error::*;

use self::framebuffer::make_framebuffers;
use self::types::Position;

/// Holds the entire state of Hexil's rendering system. It probably doesn't make sense to ever make more than one of this, but technically nothing is stopping you.
#[allow(dead_code)]
pub struct Renderer {
    instance: Arc<vk::instance::Instance>,
    window: Arc<winit::window::Window>,
    surface: Arc<vk::swapchain::Surface>,
    physical_device: Arc<vk::device::physical::PhysicalDevice>,
    logical_device: Arc<vk::device::Device>,
    command_allocator: vk::command_buffer::allocator::StandardCommandBufferAllocator,
    graphics_queue: Arc<vk::device::Queue>,
    transfer_queue: Arc<vk::device::Queue>,
    allocator: Arc<vk::memory::allocator::StandardMemoryAllocator>,
}

mod vert {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/canvas_vert.glsl",
    }
}
mod frag {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/canvas_frag.glsl",
    }
}

impl Renderer {
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    fn make_pipeline(
        &self,
        vert: Arc<vk::shader::ShaderModule>,
        frag: Arc<vk::shader::ShaderModule>,
        render_pass: Arc<vk::render_pass::RenderPass>,
        viewport: &Viewport,
    ) -> Result<Arc<vk::pipeline::GraphicsPipeline>, RendererError> {
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one.
        let vs = vert
            .entry_point("main")
            .ok_or(RendererError::ShaderSourceNotFound)?;
        let fs = frag
            .entry_point("main")
            .ok_or(RendererError::ShaderSourceNotFound)?;

        let vertex_input_state = Position::per_vertex().definition(&vs.info().input_interface)?;

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            self.logical_device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(self.logical_device.clone())?,
        )?;

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        Ok(GraphicsPipeline::new(
            self.logical_device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                // The stages of our pipeline, we have vertex and fragment stages.
                stages: stages.into_iter().collect(),
                // Describes the layout of the vertex input and how should it behave.
                vertex_input_state: Some(vertex_input_state),
                // Indicate the type of the primitives (the default is a list of triangles).
                input_assembly_state: Some(InputAssemblyState::default()),
                // Set the fixed viewport.
                viewport_state: Some(ViewportState {
                    viewports: [viewport.clone()].into(),
                    ..Default::default()
                }),
                // Ignore these for now.
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                // This graphics pipeline object concerns the first pass of the render pass.
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )?)
    }
}

/// A command that can be sent to the main render thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderCommand {
    Redraw,
    /// The renderer must handle the window being resized to the new dimensions (in physical pixels).
    WindowResized([u32; 2]),
    /// The renderer should shut down gracefully
    Shutdown,
}

struct SwapchainWrapper {
    viewport: Viewport,
    swapchain: Arc<vk::swapchain::Swapchain>,
    swapchain_images: Vec<Arc<vk::image::Image>>,
    render_pass: Arc<vk::render_pass::RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: PipelineWrapper,
}

impl SwapchainWrapper {
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    pub fn new(
        renderer: &Renderer,
        size: [u32; 2],
        vert: Arc<vk::shader::ShaderModule>,
        frag: Arc<vk::shader::ShaderModule>,
        vertex_buffer: Subbuffer<[Position]>,
    ) -> Result<Option<SwapchainWrapper>, RendererError> {
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: size.map(|f| f as f32),
            depth_range: 0.0..=1.0,
        };
        let (swapchain, swapchain_images): (
            Arc<vk::swapchain::Swapchain>,
            Vec<Arc<vk::image::Image>>,
        ) = match renderer.make_swapchain(None, size) {
            Ok(Some(it)) => it,
            Ok(None) => return Ok(None),
            Err(err) => return Err(err),
        };
        let render_pass: Arc<vk::render_pass::RenderPass> =
            renderer.make_renderpass(swapchain.clone())?;
        let framebuffers: Vec<Arc<Framebuffer>> =
            make_framebuffers(&swapchain_images, render_pass.clone())?;

        let pipeline = PipelineWrapper::new(
            &renderer,
            vert,
            frag,
            vertex_buffer,
            &render_pass,
            &viewport,
            &framebuffers,
        )?;

        Ok(Some(Self {
            viewport,
            swapchain,
            swapchain_images,
            render_pass,
            framebuffers,
            pipeline,
        }))
    }

    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    pub fn rebuild(
        self,
        renderer: &Renderer,
        size: [u32; 2],
    ) -> Result<Option<SwapchainWrapper>, RendererError> {
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: size.map(|f| f as f32),
            depth_range: 0.0..=1.0,
        };
        let old_format = self.swapchain.image_format();
        let (swapchain, swapchain_images): (
            Arc<vk::swapchain::Swapchain>,
            Vec<Arc<vk::image::Image>>,
        ) = match renderer.make_swapchain(Some((self.swapchain, self.swapchain_images)), size) {
            Ok(Some(it)) => it,
            Ok(None) => return Ok(None),
            Err(err) => return Err(err),
        };
        let render_pass: Arc<vk::render_pass::RenderPass> =
            if swapchain.image_format() != old_format {
                renderer.make_renderpass(swapchain.clone())?
            } else {
                self.render_pass
            };
        let framebuffers: Vec<Arc<Framebuffer>> =
            make_framebuffers(&swapchain_images, render_pass.clone())?;

        let pipeline = PipelineWrapper::new(
            &renderer,
            self.pipeline.vert,
            self.pipeline.frag,
            self.pipeline.vertex_buffer,
            &render_pass,
            &viewport,
            &framebuffers,
        )?;
        Ok(Some(Self {
            viewport,
            swapchain,
            swapchain_images,
            render_pass,
            framebuffers,
            pipeline,
        }))
    }
}

struct PipelineWrapper {
    vert: Arc<vk::shader::ShaderModule>,
    frag: Arc<vk::shader::ShaderModule>,
    vertex_buffer: vk::buffer::Subbuffer<[Position]>,
    pipeline: Arc<vk::pipeline::GraphicsPipeline>,
    command_buffers: Vec<Arc<vk::command_buffer::PrimaryAutoCommandBuffer>>,
}

impl PipelineWrapper {
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    pub fn new(
        renderer: &Renderer,
        vert: Arc<vk::shader::ShaderModule>,
        frag: Arc<vk::shader::ShaderModule>,
        vertex_buffer: Subbuffer<[Position]>,
        render_pass: &Arc<vk::render_pass::RenderPass>,
        viewport: &Viewport,
        framebuffers: &Vec<Arc<Framebuffer>>,
    ) -> Result<Self, RendererError> {
        let pipeline =
            renderer.make_pipeline(vert.clone(), frag.clone(), render_pass.clone(), &viewport)?;

        let command_buffers = command_buffers::get_command_buffers(
            &renderer.command_allocator,
            &renderer.graphics_queue.clone(),
            &pipeline,
            &framebuffers.clone(),
            &vertex_buffer,
        )?;

        Ok(Self {
            vert,
            frag,
            vertex_buffer,
            pipeline,
            command_buffers,
        })
    }
}
impl SwapchainWrapper {
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    fn make_canvas_swapchain(
        renderer: &Renderer,
        size: [u32; 2],
    ) -> Result<Option<SwapchainWrapper>, RendererError> {
        let vertex1 = Position {
            position: [-0.5, -0.5],
        };
        let vertex2 = Position {
            position: [0.0, 0.5],
        };
        let vertex3 = Position {
            position: [0.5, -0.25],
        };

        let vertex_buffer = vk::buffer::Buffer::from_iter(
            renderer.allocator.clone(),
            vk::buffer::BufferCreateInfo {
                usage: vk::buffer::BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            vk::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vk::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vec![vertex1, vertex2, vertex3],
        )?;

        let vert: Arc<vk::shader::ShaderModule> = vert::load(renderer.logical_device.clone())?;
        let frag: Arc<vk::shader::ShaderModule> = frag::load(renderer.logical_device.clone())?;

        Ok(SwapchainWrapper::new(
            &renderer,
            size,
            vert.clone(),
            frag.clone(),
            vertex_buffer,
        )?)
    }
}

/// Runs Hexil's rendering system. Should be run in it's own dedicated OS thread.
#[instrument(skip_all)]
pub fn render_thread(
    window: Arc<Window>,
    render_command_channel: std::sync::mpsc::Receiver<RenderCommand>,
) -> Result<(), renderer_error::RendererError> {
    use try_log::try_or_err;
    window.set_visible(true);
    let renderer = Renderer::new(window.clone());
    if let Err(e) = renderer {
        error!("Failed to init renderer! {}", e);
        return Err(e);
    }
    let renderer = try_or_err!(renderer);

    let mut swapchain_wrapper = try_or_err!(SwapchainWrapper::make_canvas_swapchain(
        &renderer,
        window.inner_size().into()
    ));

    loop {
        match render_command_channel.recv() {
            Ok(RenderCommand::Redraw) => {
                if let Some(swapchain_wrapper) = &swapchain_wrapper {
                    let (image_i, suboptimal, acquire_future) =
                        match vk::swapchain::acquire_next_image(
                            swapchain_wrapper.swapchain.clone(),
                            None,
                        )
                        .map_err(Validated::unwrap)
                        {
                            Ok(r) => r,
                            Err(VulkanError::OutOfDate) => {
                                continue;
                            }
                            Err(e) => return Err(e.into()),
                        };

                    let execution = try_or_err!(vk::sync::now(renderer.logical_device.clone())
                        .join(acquire_future)
                        .then_execute(
                            renderer.graphics_queue.clone(),
                            swapchain_wrapper.pipeline.command_buffers[image_i as usize].clone(),
                        ))
                    .then_swapchain_present(
                        renderer.graphics_queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(
                            swapchain_wrapper.swapchain.clone(),
                            image_i,
                        ),
                    )
                    .then_signal_fence_and_flush();
                    if let Ok(execution) = execution {
                        try_or_err!(execution.wait(None));
                    } else {
                        error!("Oh no!");
                    }
                }
            }
            Err(e) => {
                error!("Inter-thread communication failure! {}", e);
                return Err(e.into());
            }
            Ok(RenderCommand::WindowResized(new_size)) => {
                swapchain_wrapper = try_or_err!(if swapchain_wrapper.is_some() {
                    swapchain_wrapper.unwrap().rebuild(&renderer, new_size)
                } else {
                    SwapchainWrapper::make_canvas_swapchain(&renderer, new_size)
                });
            }
            Ok(RenderCommand::Shutdown) => return Ok(()),
        }
    }
}
