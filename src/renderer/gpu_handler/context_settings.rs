use super::device_select;
use std::sync::Arc;
use vulkano as vk;
use vulkano_util as vk_util;

use super::Renderer;

impl Renderer {
    pub(super) fn get_context() -> vk_util::context::VulkanoContext {
        let opts = vk_util::context::VulkanoConfig {
            instance_create_info: vk::instance::InstanceCreateInfo::application_from_cargo_toml(),
            debug_create_info: None,
            device_filter_fn: Arc::new(device_select::device_filter),
            device_priority_fn: Arc::new(device_select::device_rank),
            device_extensions: vk::device::DeviceExtensions {
                ..Default::default()
            },
            device_features: vk::device::Features {
                ..Default::default()
            },
            print_device_name: true,
        };
        vk_util::context::VulkanoContext::new(opts)
    }
}
