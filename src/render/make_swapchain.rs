use std::sync::Arc;

use super::renderer_error;
use super::Renderer;
use tracing::instrument;
use try_log::log_tries;
use vk::swapchain::ColorSpace;
use vulkano as vk;

impl Renderer {
    /// Wraps the process of building a new swapchain for a window.
    pub(crate) fn make_swapchain(
        &self,
        old_swapchain: Option<(Arc<vk::swapchain::Swapchain>, Vec<Arc<vk::image::Image>>)>,
        new_size: [u32; 2],
    ) -> Result<
        Option<(Arc<vk::swapchain::Swapchain>, Vec<Arc<vk::image::Image>>)>,
        renderer_error::RendererError,
    > {
        let _guard = tracing::info_span!("make_swapchain").entered();
        if new_size == [0u32, 0u32] {
            Ok(None)
        } else if let Some(swapchain) = old_swapchain {
            let mut create_info = swapchain.0.create_info();
            create_info.image_extent = new_size;
            Ok(Some(swapchain.0.recreate(create_info)?))
        } else {
            let present_mode = if self
                .physical_device
                .surface_present_modes(self.surface.as_ref(), Default::default())
                .is_ok_and(|mut a| a.any(|b| b == vk::swapchain::PresentMode::Mailbox))
            {
                vk::swapchain::PresentMode::Mailbox
            } else {
                vk::swapchain::PresentMode::Fifo
            };
            let scaling_behavior = if self
                .logical_device
                .enabled_extensions()
                .ext_swapchain_maintenance1
            {
                Some(vk::swapchain::PresentScaling::AspectRatioStretch)
            } else {
                None
            };
            let swapchain = vk::swapchain::SwapchainCreateInfo {
                scaling_behavior,
                image_format: self
                    .physical_device
                    .surface_formats(&self.surface, Default::default())?
                    .into_iter()
                    .find(|(_, c)| ColorSpace::SrgbNonLinear == *c)
                    .unwrap()
                    .0,
                image_view_formats: Default::default(),
                image_extent: new_size,
                image_usage: vk::image::ImageUsage::COLOR_ATTACHMENT, // TODO: Might need to be updated to allow for displaying
                pre_transform: vk::swapchain::SurfaceTransform::Identity, // TODO: Switch to inherit from OS
                composite_alpha: vk::swapchain::CompositeAlpha::Opaque,
                present_mode,
                // present_modes: todo!(), // TODO: Add support for changing this in a settings menu
                ..Default::default()
            };
            Ok(Some(vk::swapchain::Swapchain::new(
                self.logical_device.clone(),
                self.surface.clone(),
                swapchain,
            )?))
        }
    }
}
