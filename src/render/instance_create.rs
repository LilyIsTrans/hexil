use std::sync::Arc;

use try_log::log_tries;
use vulkano as vk;

use vk::VulkanLibrary;

use tracing::instrument;

use super::renderer_error;

use super::Renderer;

impl Renderer {
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    /// Wraps the process of acquiring a Vulkan instance.
    pub fn get_instance(
        lib: Arc<VulkanLibrary>,
        window: Arc<winit::window::Window>,
    ) -> Result<Arc<vk::instance::Instance>, renderer_error::RendererError> {
        let needed_extensions = vk::swapchain::Surface::required_extensions(window.as_ref());

        let needed_extensions = vk::instance::InstanceExtensions {
            khr_get_surface_capabilities2: true,
            khr_get_physical_device_properties2: true,
            ..Default::default()
        }
        .union(&needed_extensions);

        let mut info = vk::instance::InstanceCreateInfo::application_from_cargo_toml();
        info.enabled_extensions = needed_extensions;

        Ok(vk::instance::Instance::new(lib, info)?)
    }
}
