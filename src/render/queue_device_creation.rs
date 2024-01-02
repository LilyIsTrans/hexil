use std::sync::Arc;

use super::renderer_error;
use tracing::error;
use tracing::instrument;
use try_log::log_tries;
use vulkano as vk;

use tracing::warn;

use super::Renderer;

impl Renderer {
    /// On success, returns a tuple `(device, transfer_queue, graphics_queue)`.
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
    pub(crate) fn get_queues_and_device(
        physical_device: Arc<vk::device::physical::PhysicalDevice>,
    ) -> Result<
        (
            Arc<vk::device::Device>,
            Arc<vk::device::Queue>,
            Arc<vk::device::Queue>,
        ),
        renderer_error::RendererError,
    > {
        let graphics_queue_flags = vk::device::QueueFlags::GRAPHICS;
        let transfer_queue_flags = vk::device::QueueFlags::TRANSFER;
        let both = graphics_queue_flags.union(transfer_queue_flags);
        let queue_family_indices = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .filter(|(_, p)| p.queue_flags.intersects(both));
        let graphics_queue_families: Vec<usize> = queue_family_indices
            .clone()
            .filter(|(_, p)| p.queue_flags.contains(graphics_queue_flags))
            .map(|(i, _)| i)
            .collect();
        let transfer_queue_families: Vec<usize> = queue_family_indices
            .filter(|(_, p)| p.queue_flags.contains(transfer_queue_flags))
            .map(|(i, _)| i)
            .collect();

        let (both, graphics_only): (Vec<usize>, Vec<usize>) = graphics_queue_families
            .iter()
            .partition(|i| transfer_queue_families.contains(i));
        let transfer_only: Vec<usize> = transfer_queue_families
            .iter()
            .filter(|i| !graphics_only.contains(i))
            .copied()
            .collect();
        // Selects a graphics queue family and a transfer queue family.
        // If possible, it will select different queue families.
        let (graphics_family, transfer_family) =
            match (both.len(), graphics_only.len(), transfer_only.len()) {
                (0, 0, _) => {
                    error!("No graphics queues!");
                    return Err(renderer_error::RendererError::NoGraphicsQueues);
                }
                (0, _, 0) => {
                    error!("No transfer queues!");
                    return Err(renderer_error::RendererError::NoTransferQueues);
                }
                (1, 0, 0) => {
                    warn!("Only one queue available, performance may be affected.");
                    let q = both
                        .first()
                        .expect("We just confirmed that both has exactly 1 element.");
                    (*q, *q)
                }
                (_, 0, 0) => (both[0], both[1]),
                (_, 0, _) => (both[0], transfer_only[0]),
                (_, _, 0) => (graphics_only[0], both[0]),
                (_, _, _) => (graphics_only[0], transfer_only[0]),
            };

        let mut queues = vec![0.5];
        if graphics_family == transfer_family {
            queues.push(0.5);
        }
        let graphics_queue_create_info = vk::device::QueueCreateInfo {
            queue_family_index: graphics_family
                .try_into()
                .expect("I got this index from this device. It better be able to take it back."),
            queues,
            ..Default::default()
        };

        let transfer_queue_create_info = vk::device::QueueCreateInfo {
            queue_family_index: transfer_family
                .try_into()
                .expect("I got this index from this device. It better be able to take it back."),
            ..Default::default()
        };

        let mut queue_create_infos = Vec::<vk::device::QueueCreateInfo>::with_capacity(2);

        queue_create_infos.push(graphics_queue_create_info);
        if graphics_family != transfer_family {
            queue_create_infos.push(transfer_queue_create_info);
        };

        let logical_device = vk::device::DeviceCreateInfo {
            queue_create_infos,
            enabled_extensions: physical_device
                .supported_extensions()
                .intersection(&super::consts::ALL_KHR_DEVICE_EXTENSIONS),
            // enabled_features: Features {
            //     triangle_fans: true,
            //     ..Default::default()
            // },
            ..Default::default()
        };

        let (device, queues) = vk::device::Device::new(physical_device.clone(), logical_device)?;

        let queues = queues.collect::<Arc<[_]>>();
        let graphics_queue: Arc<vk::device::Queue> = queues
            .iter()
            .find(|q| {
                graphics_family
                    == q.queue_family_index()
                        .try_into()
                        .expect("I sure hope u32 fits into usize.")
            })
            .expect("If it didn't exist, we'd have returned an error a few lines ago.")
            .clone();
        let transfer_queue: Arc<vk::device::Queue> = queues
            .iter()
            .find(|q| {
                transfer_family
                    == q.queue_family_index()
                        .try_into()
                        .expect("I sure hope u32 fits into usize.")
                    && q.id_within_family() != graphics_queue.id_within_family()
            })
            .expect("If it didn't exist, we'd have returned an error a few lines ago.")
            .clone();

        Ok((device, transfer_queue, graphics_queue))
    }
}
