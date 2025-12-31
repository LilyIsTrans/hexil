#include <SDL3/SDL_video.h>
#include <vulkan/vulkan.h>
#include <vulkan/vulkan_core.h>


typedef struct {
  SDL_Window* window;
  VkSurfaceKHR surface;
} HexilOsWindow;

struct HexilVulkanState {
  VkInstance instance;
  
};

struct HexilGlobalState  {
  HexilOsWindow main_window;
  struct HexilVulkanState vulkan_state;
};
