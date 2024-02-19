use std::sync::Arc;
use vulkano as vk;
use vulkano_util as vk_util;
pub struct Renderer {
    context: vk_util::context::VulkanoContext,
}

mod device_select;

mod context_settings;
