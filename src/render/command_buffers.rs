use smallvec::smallvec;
use tracing::instrument;
use vk::command_buffer::SubpassEndInfo;

use vk::command_buffer::SubpassContents;

use vk::command_buffer::SubpassBeginInfo;

use vk::command_buffer::RenderPassBeginInfo;

use vk::command_buffer::CommandBufferUsage;

use vk::command_buffer::AutoCommandBufferBuilder;
use vk::command_buffer::PrimaryAutoCommandBuffer;
use vk::pipeline::graphics::viewport::Viewport;
use vk::pipeline::Pipeline;
use vulkano as vk;

use super::types::Position;
use super::RendererError;

use vk::buffer::Subbuffer;

use vk::render_pass::Framebuffer;

use vk::pipeline::GraphicsPipeline;

use vk::device::Queue;

use std::sync::Arc;

use vk::command_buffer::allocator::StandardCommandBufferAllocator;

pub struct CommandBufferManager {
    pub(crate) drawing: Vec<Arc<PrimaryAutoCommandBuffer>>,
    pub(crate) transfer: Arc<PrimaryAutoCommandBuffer>,
}
impl CommandBufferManager {
    #[instrument(skip_all, err)]
    pub(crate) fn new(
        command_buffer_allocator: &StandardCommandBufferAllocator,
        gfx_queue: &Arc<Queue>,
        transfer_queue: &Arc<Queue>,
        pipeline: &Arc<GraphicsPipeline>,
        viewport: Viewport,
        framebuffers: &Vec<Arc<Framebuffer>>,
        vertex_buffer: &Subbuffer<[Position]>,
        manager: &super::canvas_manager::CanvasBuffersManager,
    ) -> Result<Self, RendererError> {
        let drawing_buffers = framebuffers
            .iter()
            .map(|framebuffer| {
                let mut builder = AutoCommandBufferBuilder::primary(
                    command_buffer_allocator,
                    gfx_queue.queue_family_index(),
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
                    .set_viewport(0, smallvec![viewport.clone()])?
                    .bind_descriptor_sets(
                        vk::pipeline::PipelineBindPoint::Graphics,
                        pipeline.layout().clone(),
                        0,
                        manager.descriptors.clone().unwrap(),
                    )?
                    .draw(vertex_buffer.len() as u32, 300, 0, 0)?
                    .end_render_pass(SubpassEndInfo::default())?;

                Ok(builder.build()?)
            })
            .collect::<Result<Vec<_>, RendererError>>()?;

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            transfer_queue.queue_family_index(),
            // Don't forget to write the correct buffer usage.
            CommandBufferUsage::MultipleSubmit,
        )?;
        builder
            .copy_buffer(vk::command_buffer::CopyBufferInfo::buffers(
                manager.canvas_settings_host.clone(),
                manager.canvas_settings_device.clone(),
            ))?
            .copy_buffer(vk::command_buffer::CopyBufferInfo::buffers(
                manager.canvas_indices_host.clone(),
                manager.canvas_indices_device.clone(),
            ))?;
        Ok(Self {
            drawing: drawing_buffers,
            transfer: builder.build()?,
        })
    }
}
