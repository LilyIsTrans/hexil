use tracing::instrument;
use try_log::log_tries;
use vk::command_buffer::SubpassEndInfo;

use vk::command_buffer::SubpassContents;

use vk::command_buffer::SubpassBeginInfo;

use vk::command_buffer::RenderPassBeginInfo;

use vk::command_buffer::CommandBufferUsage;

use vk::command_buffer::AutoCommandBufferBuilder;
use vk::command_buffer::PrimaryAutoCommandBuffer;
use vulkano as vk;

use super::types::Position;
use super::RendererError;

use vk::buffer::Subbuffer;

use vk::render_pass::Framebuffer;

use vk::pipeline::GraphicsPipeline;

use vk::device::Queue;

use std::sync::Arc;

use vk::command_buffer::allocator::StandardCommandBufferAllocator;

#[instrument(skip_all)]
#[log_tries(tracing::error)]
pub(crate) fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    vertex_buffer: &Subbuffer<[Position]>,
) -> Result<Vec<Arc<PrimaryAutoCommandBuffer>>, RendererError> {
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
