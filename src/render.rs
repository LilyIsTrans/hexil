mod buffer_make;
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

use tracing::info;
use vk::command_buffer::allocator::StandardCommandBufferAllocator;
use vk::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo,
    SubpassContents, SubpassEndInfo,
};
use vk::device::Queue;
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
use vk::{buffer::Subbuffer, command_buffer::PrimaryCommandBufferAbstract, sync::GpuFuture};
use vk::{Validated, VulkanError};
use vulkano as vk;
use winit::window::Window;

mod types;

mod consts;

mod renderer_error;
pub use renderer_error::*;

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
    render_pass: Option<Arc<vk::render_pass::RenderPass>>,
    framebuffers: Option<Vec<Arc<Framebuffer>>>,
    vertex_buffer: vk::buffer::Subbuffer<[Position]>,
    swapchain: Option<(Arc<vk::swapchain::Swapchain>, Vec<Arc<vk::image::Image>>)>,
    allocator: Arc<vk::memory::allocator::StandardMemoryAllocator>,
}

/// A command that can be sent to the main render thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderCommand {
    /// The renderer must handle the window being resized to the new dimensions (in physical pixels).
    WindowResized([u32; 2]),
    /// The renderer should shut down gracefully
    Shutdown,
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

use vulkano::command_buffer::PrimaryAutoCommandBuffer;
type Res<T> = Result<T, RendererError>;
fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    vertex_buffer: &Subbuffer<[Position]>,
) -> Res<Vec<Arc<PrimaryAutoCommandBuffer>>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                // Don't forget to write the correct buffer usage.
                CommandBufferUsage::MultipleSubmit,
            )?;

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )?
                .bind_pipeline_graphics(pipeline.clone())?
                .bind_vertex_buffers(0, vertex_buffer.clone())?
                .draw(vertex_buffer.len() as u32, 1, 0, 0)?
                .end_render_pass(SubpassEndInfo::default())?;

            Ok(builder.build()?)
        })
        .collect()
}

/// Runs Hexil's rendering system. Should be run in it's own dedicated OS thread.
#[instrument]
pub fn render_thread(
    window: Arc<Window>,
    render_command_channel: std::sync::mpsc::Receiver<RenderCommand>,
) -> Result<(), renderer_error::RendererError> {
    let size = window.inner_size();
    let renderer = Renderer::new(window.clone());
    if let Err(e) = renderer {
        error!("Failed to init renderer! {}", e);
        return Err(e);
    }
    let mut renderer = renderer?;
    renderer.window.set_visible(true);
    render_command_channel.recv_timeout(Duration::from_secs(1));
    renderer.make_swapchain(size.into())?;
    renderer.make_renderpass()?;
    renderer.make_framebuffers()?;

    let vert = vert::load(renderer.logical_device.clone())?;
    let frag = frag::load(renderer.logical_device.clone())?;

    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: window.inner_size().into(),
        depth_range: 0.0..=1.0,
    };

    let pipeline = {
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
            renderer.logical_device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(renderer.logical_device.clone())?,
        )?;

        let subpass = Subpass::from(renderer.render_pass.as_ref().unwrap().clone(), 0).unwrap();

        GraphicsPipeline::new(
            renderer.logical_device.clone(),
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
                    viewports: [viewport].into_iter().collect(),
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
        )?
    };

    let command_buffers = get_command_buffers(
        &renderer.command_allocator,
        &renderer.graphics_queue.clone(),
        &pipeline,
        &renderer.framebuffers.as_ref().clone().unwrap(),
        &renderer.vertex_buffer,
    )?;

    loop {
        match render_command_channel.recv_timeout(Duration::from_nanos(100)) {
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let (image_i, suboptimal, acquire_future) = match vk::swapchain::acquire_next_image(
                    renderer.swapchain.as_ref().unwrap().clone().0,
                    None,
                )
                .map_err(Validated::unwrap)
                {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        panic!("failed to acquire next image")
                    }
                    Err(e) => panic!("failed to acquire next image: {e}"),
                };

                let execution = vk::sync::now(renderer.logical_device.clone())
                    .join(acquire_future)
                    .then_execute(
                        renderer.graphics_queue.clone(),
                        command_buffers[image_i as usize].clone(),
                    )?
                    .then_swapchain_present(
                        renderer.graphics_queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(
                            renderer.swapchain.as_ref().unwrap().0.clone(),
                            image_i,
                        ),
                    )
                    .then_signal_fence_and_flush()?
                    .wait(None);
            }
            Err(e) => {
                error!("Inter-thread communication failure! {}", e);
                return Err(e.into());
            }
            Ok(RenderCommand::WindowResized(new_size)) => renderer.make_swapchain(new_size)?,
            Ok(RenderCommand::Shutdown) => return Ok(()),
        }
    }
}
