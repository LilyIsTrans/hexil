use tracing::instrument;
use vk::{
    image::view::ImageView,
    render_pass::{Framebuffer, FramebufferCreateInfo},
};
use vulkano as vk;

impl super::Renderer {
    #[instrument(skip(self))]
    pub fn make_framebuffers(&mut self) -> Result<(), super::RendererError> {
        if let Some(swpchain) = self.swapchain.clone() {
            self.framebuffers = Some(
                swpchain
                    .1
                    .iter()
                    .map(|image| {
                        let view = ImageView::new_default(image.clone()).unwrap();
                        Framebuffer::new(
                            self.render_pass.clone().expect(
                                "If we have a swapchain, we should always also have a renderpass",
                            ),
                            FramebufferCreateInfo {
                                attachments: vec![view],
                                ..Default::default()
                            },
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }

        Ok(())
    }
}
