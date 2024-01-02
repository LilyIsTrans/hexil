use std::sync::Arc;

use super::renderer_error;
use super::Renderer;
use tracing::instrument;
use try_log::log_tries;
use vk::swapchain::ColorSpace;
use vulkano as vk;

impl Renderer {
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    /// Wraps the process of building a new swapchain for a window.
    pub(crate) fn make_swapchain(
        &self,
        old_swapchain: Option<(Arc<vk::swapchain::Swapchain>, Vec<Arc<vk::image::Image>>)>,
        new_size: [u32; 2],
    ) -> Result<
        Option<(Arc<vk::swapchain::Swapchain>, Vec<Arc<vk::image::Image>>)>,
        renderer_error::RendererError,
    > {
        if new_size == [0u32, 0u32] {
            Ok(None)
        } else if let Some(swapchain) = old_swapchain {
            let mut create_info = swapchain.0.create_info();
            create_info.image_extent = new_size;
            Ok(Some(swapchain.0.recreate(create_info)?))
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
            Ok(Some(vk::swapchain::Swapchain::new(
                self.logical_device.clone(),
                self.surface.clone(),
                swapchain,
            )?))
        }
    }
}
