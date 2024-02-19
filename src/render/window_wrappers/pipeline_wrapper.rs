use crate::render::canvas_manager::CanvasBuffersManager;

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
    pub(crate) vertex_buffer: vk::buffer::Subbuffer<[Position]>,
    pub(crate) pipeline: Arc<vk::pipeline::GraphicsPipeline>,
    pub(crate) command_buffers: crate::render::command_buffers::CommandBufferManager,
}

impl PipelineWrapper {
    #[instrument(skip_all, err)]
    #[log_tries(tracing::error)]
    pub fn new(
        renderer: &Renderer,
        vert: Arc<vk::shader::ShaderModule>,
        frag: Arc<vk::shader::ShaderModule>,
        vertex_buffer: Subbuffer<[Position]>,
        render_pass: &Arc<vk::render_pass::RenderPass>,
        viewport: Viewport,
        framebuffers: &Vec<Arc<Framebuffer>>,
        manager: &CanvasBuffersManager,
    ) -> Result<Self, RendererError> {
        let pipeline =
            renderer.make_pipeline(vert.clone(), frag.clone(), render_pass.clone(), &viewport)?;

        let command_buffers = crate::render::command_buffers::CommandBufferManager::new(
            &renderer.command_allocator,
            &renderer.graphics_queue,
            &renderer.transfer_queue,
            &pipeline,
            viewport,
            &framebuffers,
            &vertex_buffer,
            manager,
        )?;

        Ok(Self {
            vertex_buffer,
            pipeline,
            command_buffers,
        })
    }

    #[instrument(skip_all, err)]
    pub fn rebuild(
        self,
        renderer: &Renderer,
        viewport: Viewport,
        framebuffers: &Vec<Arc<Framebuffer>>,
        manager: &CanvasBuffersManager,
    ) -> Result<Self, RendererError> {
        let command_buffers = crate::render::command_buffers::CommandBufferManager::new(
            &renderer.command_allocator,
            &renderer.graphics_queue,
            &renderer.transfer_queue,
            &self.pipeline,
            viewport,
            &framebuffers,
            &self.vertex_buffer,
            manager,
        )?;

        Ok(Self {
            command_buffers,
            ..self
        })
    }
}
