pub use vulkanalia::prelude::v1_4::*;
pub use vulkanalia::vk::KhrGetSurfaceCapabilities2ExtensionInstanceCommands;
pub use vulkanalia::vk::KhrPresentWait2ExtensionDeviceCommands;
pub use vulkanalia::vk::KhrSurfaceExtensionInstanceCommands;
pub use vulkanalia::vk::KhrSwapchainExtensionDeviceCommands;
pub use vulkanalia::vk::KhrSwapchainExtensionInstanceCommands;
pub use vulkanalia::vk::KhrSwapchainMaintenance1ExtensionDeviceCommands;

pub const REQUIRED_EXTENSIONS: &'static [vulkanalia::vk::Extension] = &[
    vulkanalia::vk::KHR_SURFACE_EXTENSION,
    vulkanalia::vk::KHR_SWAPCHAIN_EXTENSION,
    vulkanalia::vk::KHR_GET_SURFACE_CAPABILITIES2_EXTENSION,
    vulkanalia::vk::KHR_PRESENT_ID2_EXTENSION,
    vulkanalia::vk::KHR_PRESENT_WAIT2_EXTENSION,
    vulkanalia::vk::KHR_SWAPCHAIN_MAINTENANCE1_EXTENSION,
    vulkanalia::vk::KHR_SWAPCHAIN_MUTABLE_FORMAT_EXTENSION,
];

pub enum UniqueVulkanId {
    LUID(vulkanalia::vk::ByteArray<{ vulkanalia::vk::LUID_SIZE }>),
    UUID(vulkanalia::vk::ByteArray<{ vulkanalia::vk::UUID_SIZE }>),
}
