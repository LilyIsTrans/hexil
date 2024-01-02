use std::sync::Arc;

use tracing::instrument;
use try_log::log_tries;
use vulkano as vk;

type Result<T> = std::result::Result<T, super::RendererError>;
impl super::Renderer {
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    pub(super) fn make_renderpass(
        &self,
        swapchain: Arc<vk::swapchain::Swapchain>,
    ) -> Result<Arc<vk::render_pass::RenderPass>> {
        Ok(vulkano::single_pass_renderpass!(
            self.logical_device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )?)
    }
}
