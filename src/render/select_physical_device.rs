use std::sync::Arc;

use super::renderer_error;

use vulkano as vk;

use tracing::info;
use tracing::instrument;

use super::Renderer;

impl Renderer {
    /// Selects a Vulkan physical device. Currently, it does this by selecting whichever can do the most simultaneous instanced draws, but this is a crude heuristic. It should be updated later.
    #[instrument(skip_all)]
    pub(crate) fn get_physical_device(
        instance: Arc<vk::instance::Instance>,
    ) -> Result<Arc<vk::device::physical::PhysicalDevice>, renderer_error::RendererError> {
        let physical_device = instance
            .enumerate_physical_devices()?
            .inspect(|dev| info!("Physical Device detected: {}", dev.properties().device_name))
            .max_by_key(|dev| dev.properties().max_instance_count) // TODO: Decide which device to use by a more sophisticated method than simply whichever can have the most instances
            .ok_or(renderer_error::RendererError::NoPhysicalDevices)?;

        info!(
            "Selected Physical Device: {}",
            physical_device.properties().device_name
        );
        Ok(physical_device)
    }
}
