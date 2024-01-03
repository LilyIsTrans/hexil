use super::super::RendererError;

use tracing::instrument;
use try_log::log_tries;
use vulkano as vk;

use vk::render_pass::Framebuffer;

use vk::pipeline::graphics::viewport::Viewport;

use super::super::Renderer;

use super::super::types::Position;

use vk::buffer::Subbuffer;

use std::sync::Arc;

pub(in crate::render) struct PipelineWrapper {
    pub(crate) vert: Arc<vk::shader::ShaderModule>,
    pub(crate) frag: Arc<vk::shader::ShaderModule>,
    pub(crate) vertex_buffer: vk::buffer::Subbuffer<[Position]>,
    pub(crate) pipeline: Arc<vk::pipeline::GraphicsPipeline>,
    pub(crate) command_buffers: Vec<Arc<vk::command_buffer::PrimaryAutoCommandBuffer>>,
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
        viewport: Viewport,
        framebuffers: &Vec<Arc<Framebuffer>>,
    ) -> Result<Self, RendererError> {
        let pipeline =
            renderer.make_pipeline(vert.clone(), frag.clone(), render_pass.clone(), &viewport)?;

        let command_buffers = crate::render::command_buffers::get_command_buffers(
            &renderer.command_allocator,
            &renderer.graphics_queue.clone(),
            &pipeline,
            viewport,
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
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    pub fn rebuild(
        self,
        renderer: &Renderer,
        viewport: Viewport,
        framebuffers: &Vec<Arc<Framebuffer>>,
    ) -> Result<Self, RendererError> {
        let command_buffers = crate::render::command_buffers::get_command_buffers(
            &renderer.command_allocator,
            &renderer.graphics_queue.clone(),
            &self.pipeline,
            viewport,
            &framebuffers.clone(),
            &self.vertex_buffer,
        )?;

        Ok(Self {
            command_buffers,
            ..self
        })
    }
}
