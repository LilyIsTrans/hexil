#include <SDL3/SDL.h>
#include "hexil.h"
#include "dcimgui.h"
#include "dcimgui_impl_sdl3.h"
#include "dcimgui_impl_vulkan.h"
#include <volk.h>


void SDL_AppQuit(struct HexilGlobalState* appstate, SDL_AppResult result) {
    vkDeviceWaitIdle(appstate->vulkan_state.device);
    cImGui_ImplSDL3_Shutdown();
    cImGui_ImplVulkan_Shutdown();
    ImGui_DestroyContext(NULL);

    
    hexil_all_windows(appstate, &hexil_cleanup_window);
    SDL_Quit();
    return;
}
