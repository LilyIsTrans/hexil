use std::cmp::Ordering;
use std::sync::Arc;

use super::renderer_error;

use palette::chromatic_adaptation::AdaptInto;
use try_log::log_tries;
use vulkano as vk;

use tracing::info;
use tracing::instrument;

use super::Renderer;

pub(in crate::render) static MAILBOX_MODE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

fn rank_physical_devices(
    a: &(
        Arc<vk::device::physical::PhysicalDevice>,
        &vk::swapchain::Surface,
    ),
    b: &(
        Arc<vk::device::physical::PhysicalDevice>,
        &vk::swapchain::Surface,
    ),
) -> Ordering {
    match (
        a.0.surface_present_modes(a.1, vk::swapchain::SurfaceInfo::default()),
        (b.0.surface_present_modes(b.1, vk::swapchain::SurfaceInfo::default())),
    ) {
        (Ok(mut amodes), Ok(mut bmodes)) => {
            match (
                amodes.any(|f| f == vk::swapchain::PresentMode::Mailbox),
                bmodes.any(|f| f == vk::swapchain::PresentMode::Mailbox),
            ) {
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (_, _) => Ordering::Equal, // TODO: Add more checks
            }
        }
        (Ok(_), Err(_)) => Ordering::Greater,
        (Err(_), Ok(_)) => Ordering::Less,
        (Err(_), Err(_)) => Ordering::Equal,
    }
}

impl Renderer {
    /// Selects a Vulkan physical device. Currently, it does this by selecting whichever can do the most simultaneous instanced draws, but this is a crude heuristic. It should be updated later.
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    pub(crate) fn get_physical_device(
        instance: Arc<vk::instance::Instance>,
        surface: &vk::swapchain::Surface,
    ) -> Result<Arc<vk::device::physical::PhysicalDevice>, renderer_error::RendererError> {
        let physical_device = instance
            .enumerate_physical_devices()?
            .inspect(|dev| info!("Physical Device detected: {}", dev.properties().device_name))
            .map(|dev| (dev, surface))
            .max_by(rank_physical_devices)
            .map(|(dev, _)| dev)
            .ok_or(renderer_error::RendererError::NoPhysicalDevices)?;

        info!(
            "Selected Physical Device: {}",
            physical_device.properties().device_name
        );
        let _ = MAILBOX_MODE.set(
            physical_device
                .surface_present_modes(surface, Default::default())
                .is_ok_and(|mut a| a.any(|b| b == vk::swapchain::PresentMode::Mailbox)),
        );
        Ok(physical_device)
    }
}
