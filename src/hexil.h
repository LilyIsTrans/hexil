#pragma once
#include <SDL3/SDL_video.h>
#include <volk.h>


typedef struct {
  SDL_Window* window;
  VkSurfaceKHR surface;
  VkSwapchainKHR swapchain;
  VkCommandBuffer render_cmd_buffer;
  VkCommandBuffer transfer_cmd_buffer;
  VkCompositeAlphaFlagBitsKHR alpha_composite_mode;
  VkCommandPool render_cmd_pool;
  VkCommandPool transfer_cmd_pool;
} HexilOsWindow;

struct HexilVulkanState {
  VkInstance instance;
  VkPhysicalDevice physical_device;
  VkDevice device;
  
};



struct HexilGlobalState  {
  HexilOsWindow main_window;
  struct HexilVulkanState vulkan_state;
};

/// Iteratively calls [`window_callback`] on every window in the entire application
/// (except possibly certain error message or dialogue windows managed entirely by SDL),
/// setting [`is_main_window`] to true if the window happens to be the main window.
/// The main window will also be the last window to passed in.
void hexil_all_windows(struct HexilGlobalState* appstate, void (*window_callback)(struct HexilGlobalState* appstate, HexilOsWindow* window, bool is_main_window));


void hexil_cleanup_window(struct HexilGlobalState* appstate, HexilOsWindow* window, bool is_main_window);
