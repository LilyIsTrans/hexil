use std::sync::Arc;

use tracing::instrument;
use try_log::log_tries;
use vk::{
    image::view::ImageView,
    render_pass::{Framebuffer, FramebufferCreateInfo},
};
use vulkano as vk;

#[instrument(skip_all, err)]
#[log_tries(tracing::error)]
pub(super) fn make_framebuffers(
    images: &Vec<Arc<vk::image::Image>>,
    render_pass: Arc<vk::render_pass::RenderPass>,
) -> Result<Vec<Arc<Framebuffer>>, super::RendererError> {
    Ok(images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?)
}
