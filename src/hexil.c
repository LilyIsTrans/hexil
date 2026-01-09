#include <SDL3/SDL.h>
#define SDL_MAIN_USE_CALLBACKS
#include <SDL3/SDL_main.h>
#include <volk.h>
#include "hexil.h"



void hexil_cleanup_window(struct HexilGlobalState* appstate, HexilOsWindow *window, bool is_main_window) {
  SDL_assert_paranoid(vkDeviceWaitIdle != NULL);
  SDL_assert(appstate != NULL);
  SDL_assert(window != NULL);
  SDL_assert_paranoid(is_main_window ? window == &appstate->main_window : window != &appstate->main_window);

  vkDeviceWaitIdle(appstate->vulkan_state.device);
  vkDestroySwapchainKHR(appstate->vulkan_state.device, window->swapchain, NULL);
  vkDestroySurfaceKHR(appstate->vulkan_state.instance, window->surface, NULL);    
  SDL_DestroyWindow(window->window);

  memset(window, 0, sizeof(*window));
  
}


void hexil_all_windows(struct HexilGlobalState *appstate, void (*window_callback)(struct HexilGlobalState *, HexilOsWindow *, bool)) {
  window_callback(appstate, &appstate->main_window, true);
}

