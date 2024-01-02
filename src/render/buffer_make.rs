use try_log::log_tries;
use vulkano as vk;

use super::{renderer_error, Renderer};

use std::sync::Arc;

use tracing::instrument;
use vk::buffer as vbuf;

impl Renderer {
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    pub(super) fn make_buffer() -> Result<vbuf::Buffer, renderer_error::RendererError> {
        todo!()
    }
}
