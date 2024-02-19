use std::sync::Arc;

use try_log::log_tries;
use vulkano as vk;

use super::renderer_error;

use tracing::{error, instrument};

use super::Renderer;

impl Renderer {
    /// Creates a swapchain surface for a window. This is actually a thin wrapper for `vulkano::swapchain::Surface::from_window`, except that
    /// if there's an error it will fully unravel the error type and log it very cleanly. It was created because of a weird bug that has since
    /// been fixed. I don't see any reason to get rid of it though.
    #[instrument(skip_all, err)]
    #[log_tries(tracing::error)]
    pub(crate) fn get_surface(
        instance: Arc<vk::instance::Instance>,
        window: Arc<winit::window::Window>,
    ) -> Result<Arc<vk::swapchain::Surface>, renderer_error::RendererError> {
        let surface = vk::swapchain::Surface::from_window(instance.clone(), window.clone());

        if let Err(surface_error) = surface.clone() {
            match surface_error {
                vk::Validated::Error(e) => error!("Surface Creation Error: {e}"),
                vk::Validated::ValidationError(e) => {
                    error!("Surface Creation Validation Error: {e}")
                }
            };
        };
        Ok(surface?)
    }
}
