#include "hexil.h"

const char*const HEXIL_VULKAN_DEVICE_EXTENSIONS[] = {
  "VK_KHR_swapchain",
  "VK_KHR_present_id2",
  "VK_KHR_present_wait2",
  "VK_KHR_swapchain_maintenance1"
};

const char*const HEXIL_VULKAN_INSTANCE_EXTENSIONS[] = {
  "VK_KHR_surface",
  "VK_KHR_get_surface_capabilities2",
};

const VkFormat HEXIL_SWAPCHAIN_FORMATS[] = {
  VK_FORMAT_R8G8B8A8_UINT,
  VK_FORMAT_R8G8B8A8_SRGB,
  VK_FORMAT_R16G16B16A16_UNORM,
};


constexpr uint32_t HEXIL_VULKAN_DEVICE_EXTENSION_COUNT = sizeof(HEXIL_VULKAN_DEVICE_EXTENSIONS) / sizeof(typeof(HEXIL_VULKAN_DEVICE_EXTENSIONS[0]));
constexpr uint32_t HEXIL_VULKAN_INSTANCE_EXTENSION_COUNT = sizeof(HEXIL_VULKAN_INSTANCE_EXTENSIONS) / sizeof(typeof(HEXIL_VULKAN_INSTANCE_EXTENSIONS[0]));

void hexil_create_global_vulkan_instance(struct HexilGlobalState *appstate); 

void hexil_init_vulkan_device(struct HexilGlobalState* appstate);
void hexil_init_global_vulkan_state(struct HexilGlobalState* appstate);
void hexil_create_surface(struct HexilGlobalState *appstate, HexilOsWindow *window); 
void hexil_build_swapchain(struct HexilGlobalState* appstate, HexilOsWindow* window);
void hexil_allocate_window_cmd_buffers(struct HexilGlobalState *appstate, HexilOsWindow *window);
void hexil_allocate_window_cmd_pools(struct HexilGlobalState *appstate, HexilOsWindow *window);
void hexil_free_device(struct HexilGlobalState* appstate);
