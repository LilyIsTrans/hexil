mod command_buffers;
mod framebuffer;
mod init_renderer_state;
mod instance_create;
mod lib_select;
mod make_swapchain;
mod queue_device_creation;
mod render_pass;
mod select_physical_device;
mod surface_create;
mod window_wrappers;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{error, trace};

use vk::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vk::pipeline::graphics::input_assembly::InputAssemblyState;
use vk::pipeline::graphics::multisample::MultisampleState;
use vk::pipeline::graphics::rasterization::RasterizationState;
use vk::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vk::pipeline::graphics::viewport::{Viewport, ViewportState};
use vk::pipeline::graphics::GraphicsPipelineCreateInfo;
use vk::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
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

use crate::app::CanvasSize;

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

// mod vert {
//     vulkano_shaders::shader! {
//         ty: "vertex",
//         path: "src/shaders/canvas_vert.glsl",
//     }
// }
mod hex_vert {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/hex_vert.glsl",
    }
}
mod tile_frag {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/tile_frag.glsl",
    }
}
// mod frag {
//     vulkano_shaders::shader! {
//         ty: "fragment",
//         path: "src/shaders/canvas_frag.glsl",
//     }
// }

impl Renderer {
    fn make_pipeline(
        &self,
        vert: Arc<vk::shader::ShaderModule>,
        frag: Arc<vk::shader::ShaderModule>,
        render_pass: Arc<vk::render_pass::RenderPass>,
        viewport: &Viewport,
    ) -> Result<
        (
            Arc<vk::pipeline::GraphicsPipeline>,
            Arc<vk::pipeline::PipelineLayout>,
        ),
        RendererError,
    > {
        let _guard = tracing::info_span!("make_pipeline").entered();
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

        const fn check_canvas_size_type_layout() -> u32 {
            if std::mem::size_of::<CanvasSize>() % 4 != 0 {
                panic!("The size of CanvasSize must be a multiple of 4 bytes!")
            } else if std::mem::size_of::<CanvasSize>() > (u32::MAX as usize) {
                panic!("The size of CanvasSize must fit into a u32!! (Why is it even close???)")
            } else {
                std::mem::size_of::<CanvasSize>() as u32
            }
        } // By const-panicking, we trigger a compile time error!
        const CANVAS_SIZE_LAYOUT_SIZE: u32 = check_canvas_size_type_layout();
        let canvas_size_binding = vk::descriptor_set::layout::DescriptorSetLayoutBinding {
            binding_flags: vk::descriptor_set::layout::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            descriptor_count: CANVAS_SIZE_LAYOUT_SIZE,
            stages: vk::shader::ShaderStages::VERTEX,
            ..vk::descriptor_set::layout::DescriptorSetLayoutBinding::descriptor_type(
                vk::descriptor_set::layout::DescriptorType::InlineUniformBlock, // I am assuming that the entire buffer is being considered a single descriptor which is an array.
            )
        };

        let palette_binding = vk::descriptor_set::layout::DescriptorSetLayoutBinding {
            binding_flags: vk::descriptor_set::layout::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            descriptor_count: 1,
            stages: vk::shader::ShaderStages::VERTEX,
            ..vk::descriptor_set::layout::DescriptorSetLayoutBinding::descriptor_type(
                vk::descriptor_set::layout::DescriptorType::UniformTexelBuffer, // I am assuming that the entire buffer is being considered a single descriptor which is an array.
            )
        };

        let indices_binding = vk::descriptor_set::layout::DescriptorSetLayoutBinding {
            binding_flags: vk::descriptor_set::layout::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            descriptor_count: 1,
            stages: vk::shader::ShaderStages::VERTEX,
            ..vk::descriptor_set::layout::DescriptorSetLayoutBinding::descriptor_type(
                vk::descriptor_set::layout::DescriptorType::UniformTexelBuffer, // I am assuming that the entire buffer is being considered a single descriptor which is an array.
            )
        };

        let descriptors_layout = vk::descriptor_set::layout::DescriptorSetLayoutCreateInfo {
            flags:
                vk::descriptor_set::layout::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL,
            bindings: BTreeMap::from([(0, palette_binding), (1, indices_binding)]),
            ..Default::default()
        };

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
        Ok((
            GraphicsPipeline::new(
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
                    ..GraphicsPipelineCreateInfo::layout(layout.clone())
                },
            )?,
            layout,
        ))
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

/// Runs Hexil's rendering system. Should be run in it's own dedicated OS thread.
pub fn render_thread(
    window: Arc<Window>,
    render_command_channel: std::sync::mpsc::Receiver<RenderCommand>,
) -> Result<(), renderer_error::RendererError> {
    let _guard = tracing::info_span!("render_thread").entered();
    use try_log::try_or_err;
    use window_wrappers::SwapchainWrapper;
    window.set_visible(true);
    let renderer = Renderer::new(window.clone());
    if let Err(e) = renderer {
        error!("Failed to init renderer! {}", e);
        return Err(e);
    }
    let renderer = try_or_err!(renderer);

    // let CANVIEW: hex_vert::constants = hex_vert::constants {
    //     canvas_size: [2, 2],
    //     canvas_scale: 0.4.into(),
    //     canvas_offset: [0.0, 0.0],
    // };
    let mut swapchain_wrapper = try_or_err!(SwapchainWrapper::make_canvas_swapchain(
        &renderer,
        window.inner_size().into(),
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
                    swapchain_wrapper.unwrap().rebuild(&renderer, new_size)
                } else {
                    SwapchainWrapper::make_canvas_swapchain(&renderer, new_size)
                });
            }
            Ok(RenderCommand::Shutdown) => return Ok(()),
        }
    }
}
