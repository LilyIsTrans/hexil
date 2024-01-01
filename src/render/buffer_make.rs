use vulkano as vk;

use super::{renderer_error, Renderer};

use std::sync::Arc;

use tracing::instrument;
use vk::buffer as vbuf;

impl Renderer {
    #[instrument]
    pub(super) fn make_buffer() -> Result<vbuf::Buffer, renderer_error::RendererError> {
        todo!()
    }
}
