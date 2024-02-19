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
mod window_wrappers;
use std::sync::Arc;
use tracing::{error, instrument, trace};

use try_log::log_tries;
use vk::memory::allocator::MemoryAllocator;
use vk::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vk::pipeline::graphics::input_assembly::InputAssemblyState;
use vk::pipeline::graphics::multisample::MultisampleState;
use vk::pipeline::graphics::rasterization::RasterizationState;
use vk::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vk::pipeline::graphics::viewport::{Viewport, ViewportState};
use vk::pipeline::graphics::GraphicsPipelineCreateInfo;
use vk::pipeline::layout::{
    PipelineDescriptorSetLayoutCreateInfo, PipelineLayoutCreateFlags, PipelineLayoutCreateInfo,
};
use vk::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vk::render_pass::Subpass;
use vk::swapchain::SwapchainPresentInfo;
use vk::sync::GpuFuture;
use vk::{Validated, VulkanError};
use vulkano as vk;
use winit::window::Window;
mod types;

mod consts;

mod renderer_error;
pub use renderer_error::*;

use crate::render::canvas_manager::CanvasBuffersManager;

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
    descriptor_allocator: Arc<vk::descriptor_set::allocator::StandardDescriptorSetAllocator>,
}

mod vert {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/canvas_vert.glsl",
    }
}

pub(crate) mod canvas_manager;
mod frag {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/canvas_frag.glsl",
    }
}

impl Renderer {
    #[instrument(skip_all, err)]
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

        let dynamic_state = ahash::HashSet::from_iter([vk::pipeline::DynamicState::Viewport]);

        let mut input_assembly_state = InputAssemblyState::default();
        input_assembly_state.topology =
            vk::pipeline::graphics::input_assembly::PrimitiveTopology::TriangleFan;
        let input_assembly_state = Some(input_assembly_state);
        Ok(GraphicsPipeline::new(
            self.logical_device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                // The stages of our pipeline, we have vertex and fragment stages.
                stages: stages.into_iter().collect(),
                // Describes the layout of the vertex input and how should it behave.
                vertex_input_state: Some(vertex_input_state),
                // Indicate the type of the primitives (the default is a list of triangles).
                input_assembly_state,
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
                dynamic_state,
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
    CanvasSettingsChanged,
    CanvasIndicesChanged,
}

/// Runs Hexil's rendering system. Should be run in it's own dedicated OS thread.
#[instrument(skip_all, err)]
pub fn render_thread(
    window: Arc<Window>,
    render_command_channel: std::sync::mpsc::Receiver<RenderCommand>,
) -> Result<(), renderer_error::RendererError> {
    use try_log::try_or_err;
    use window_wrappers::SwapchainWrapper;
    window.set_visible(true);
    let renderer = Renderer::new(window.clone());
    if let Err(e) = renderer {
        error!("Failed to init renderer! {}", e);
        return Err(e);
    }
    let renderer = try_or_err!(renderer);
    let manager = CanvasBuffersManager::new(&renderer, 20, 15, 7)?;

    let mut swapchain_wrapper = try_or_err!(SwapchainWrapper::make_canvas_swapchain(
        &renderer,
        window.inner_size().into(),
        &manager
    ));

    loop {
        match render_command_channel.recv() {
            Ok(RenderCommand::Redraw) => {
                if let Some(swapchain_wrapper) = &swapchain_wrapper {
                    let (image_i, _suboptimal, acquire_future) =
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

                    let execution = vk::sync::now(renderer.logical_device.clone())
                        .join(acquire_future)
                        .then_execute(
                            renderer.graphics_queue.clone(),
                            swapchain_wrapper.pipeline.command_buffers.drawing[image_i as usize]
                                .clone(),
                        )?
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
                        trace!("Attempted to swap on out of date swapchain. Retrying...");
                    }
                }
            }
            Err(e) => {
                error!("Inter-thread communication failure! {}", e);
                return Err(e.into());
            }
            Ok(RenderCommand::WindowResized(new_size)) => {
                swapchain_wrapper = try_or_err!(if swapchain_wrapper.is_some() {
                    swapchain_wrapper
                        .unwrap()
                        .rebuild(&renderer, new_size, &manager)
                } else {
                    SwapchainWrapper::make_canvas_swapchain(&renderer, new_size, &manager)
                });
            }
            Ok(RenderCommand::Shutdown) => return Ok(()),
            Ok(RenderCommand::CanvasSettingsChanged) => {
                if let Some(swapchain_wrapper) = &swapchain_wrapper {
                    let execution = vk::sync::now(renderer.logical_device.clone())
                        .then_execute(
                            renderer.transfer_queue.clone(),
                            swapchain_wrapper.pipeline.command_buffers.transfer.clone(),
                        )?
                        .then_signal_semaphore_and_flush()?;
                } else {
                    tracing::warn!("The canvas settings changed, but the swapchain doesn't exist. That probably shouldn't be possible.")
                }
            }
            Ok(RenderCommand::CanvasIndicesChanged) => {
                if let Some(swapchain_wrapper) = &swapchain_wrapper {
                    let execution = vk::sync::now(renderer.logical_device.clone())
                        .then_execute(
                            renderer.transfer_queue.clone(),
                            swapchain_wrapper.pipeline.command_buffers.transfer.clone(),
                        )?
                        .then_signal_semaphore_and_flush()?;
                } else {
                    tracing::warn!("The canvas settings changed, but the swapchain doesn't exist. That probably shouldn't be possible.")
                }
            }
        }
    }
}
