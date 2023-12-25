use std::sync::Arc;

use vulkano as vk;

use super::renderer_error;

use tracing::{error, instrument};

use super::Renderer;

impl Renderer {
    #[instrument]
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
