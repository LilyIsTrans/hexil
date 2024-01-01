use super::renderer_error;
use super::Renderer;
use tracing::instrument;
use vk::swapchain::ColorSpace;
use vulkano as vk;

impl Renderer {
    #[instrument(skip(self))]
    /// Wraps the process of building a new swapchain for a window.
    pub(crate) fn make_swapchain(
        &mut self,
        new_size: [u32; 2],
    ) -> Result<(), renderer_error::RendererError> {
        if new_size == [0u32, 0u32] {
            self.swapchain = None;
        } else if self
            .swapchain
            .as_ref()
            .is_some_and(|swapchain| swapchain.0.image_extent() == new_size)
        {
            let _ = 0; // Noop to make it clearer that this is a separate do nothing path;
        } else if let Some(swapchain) = self.swapchain.clone() {
            let mut create_info = swapchain.0.create_info();
            create_info.image_extent = new_size;
            self.swapchain = Some(swapchain.0.recreate(create_info)?);
        } else {
            let swapchain = vk::swapchain::SwapchainCreateInfo {
                image_format: self
                    .physical_device
                    .surface_formats(&self.surface, Default::default())?
                    .into_iter()
                    .find(|(f, c)| ColorSpace::SrgbNonLinear == *c)
                    .unwrap()
                    .0,
                image_view_formats: Default::default(),
                image_extent: new_size,
                image_usage: vk::image::ImageUsage::STORAGE
                    | vk::image::ImageUsage::COLOR_ATTACHMENT, // TODO: Might need to be updated to allow for displaying
                pre_transform: vk::swapchain::SurfaceTransform::Identity,
                composite_alpha: vk::swapchain::CompositeAlpha::Opaque,
                present_mode: vk::swapchain::PresentMode::Fifo,
                // present_modes: todo!(), // TODO: Add support for changing this in a settings menu
                ..Default::default()
            };
            let swapchain = vk::swapchain::Swapchain::new(
                self.logical_device.clone(),
                self.surface.clone(),
                swapchain,
            )?;
            self.swapchain = Some(swapchain);
        }
        Ok(())
    }
}
