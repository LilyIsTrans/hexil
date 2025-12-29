#include <SDL3/SDL.h>
#include "hexil.h"
#include "dcimgui.h"
#include "dcimgui_impl_sdl3.h"
#include "dcimgui_impl_sdlgpu3.h"
void SDL_AppQuit(struct HexilGlobalState* appstate, SDL_AppResult result) {
  SDL_WaitForGPUIdle(appstate->gpu_device);
    cImGui_ImplSDL3_Shutdown();
    cImGui_ImplSDLGPU3_Shutdown();
    ImGui_DestroyContext(NULL);

    SDL_ReleaseWindowFromGPUDevice(appstate->gpu_device, appstate->main_window);
    SDL_DestroyGPUDevice(appstate->gpu_device);
    SDL_DestroyWindow(appstate->main_window);
    SDL_Quit();
    return;

}
