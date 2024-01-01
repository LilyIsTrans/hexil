type Result<T> = std::result::Result<T, super::RendererError>;
impl super::Renderer {
    pub(super) fn make_renderpass(&mut self) -> Result<()> {
        if let Some(swpchain) = self.swapchain.clone() {
            self.render_pass = Some(vulkano::single_pass_renderpass!(
                self.logical_device.clone(),
                attachments: {
                    color: {
                        format: swpchain.0.image_format(),
                        samples: 1,
                        load_op: Clear,
                        store_op: Store,
                    },
                },
                pass: {
                    color: [color],
                    depth_stencil: {},
                },
            )?);
        }
        Ok(())
    }
}
