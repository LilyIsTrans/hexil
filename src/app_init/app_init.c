#include <SDL3/SDL.h>
#include <SDL3/SDL_error.h>
#include <SDL3/SDL_gpu.h>
#include <SDL3/SDL_init.h>
#include <SDL3/SDL_log.h>
#include <SDL3/SDL_messagebox.h>
#include <SDL3/SDL_video.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "hexil.h"
#include "set_app_metadata.h"
#include "dcimgui.h"
#include "dcimgui_impl_sdl3.h"
#include "dcimgui_impl_sdlgpu3.h"

SDL_AppResult SDL_AppInit(struct HexilGlobalState **appstate, int argc, char **argv) {
  set_metadata();
  if (!SDL_Init(SDL_INIT_VIDEO)) {
    const char * const err = SDL_GetError();

    char manual_format_string[] =
        "Failed to initialize SDL! That's basically unrecoverable for a poor "
        "helpless automated program such as myself, so I'm probably going to "
        "close in a moment. SDL says the reason for the failure is: ";

    char* error_message = malloc(strlen(err) + sizeof(manual_format_string) + 1);

    strcpy(error_message, manual_format_string);
    strcat(error_message, err);
    strcat(error_message, "\n");

    SDL_LogError(SDL_LOG_CATEGORY_APPLICATION, "%s", error_message);

    if (!SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "Uh-oh! Failed to initialize SDL!", error_message, NULL)) {
      SDL_LogError(SDL_LOG_CATEGORY_APPLICATION, "Somewhat unsurprisingly, after failing to init, SDL also failed to spawn an error message window! In case it's at all helpful, the error message from SDL for that new failure is: %s\n", SDL_GetError());
    }

    return SDL_APP_FAILURE;
  }

  *appstate = malloc(sizeof(struct HexilGlobalState));
  memset(*appstate, 0, sizeof(struct HexilGlobalState));


  (*appstate)->main_window = SDL_CreateWindow("Hexil", 720, 480, 0);

  (*appstate)->gpu_device = SDL_CreateGPUDevice(SDL_GPU_SHADERFORMAT_SPIRV, true, NULL);

  SDL_ClaimWindowForGPUDevice((*appstate)->gpu_device, (*appstate)->main_window);

  // Setup Dear ImGui context
  CIMGUI_CHECKVERSION();
  ImGui_CreateContext(NULL);
  ImGuiIO* io = ImGui_GetIO();
  io->ConfigFlags |= ImGuiConfigFlags_NavEnableKeyboard;     // Enable Keyboard Controls
  io->ConfigFlags |= ImGuiConfigFlags_NavEnableGamepad;      // Enable Gamepad Controls
  io->ConfigFlags |= ImGuiConfigFlags_DockingEnable;         // IF using Docking Branch

  // Setup Platform/Renderer backends
  cImGui_ImplSDL3_InitForSDLGPU((*appstate)->main_window);
  ImGui_ImplSDLGPU3_InitInfo init_info = {};
  init_info.Device = (*appstate)->gpu_device;
  init_info.ColorTargetFormat = SDL_GetGPUSwapchainTextureFormat((*appstate)->gpu_device, (*appstate)->main_window);
  init_info.MSAASamples = SDL_GPU_SAMPLECOUNT_1;                      // Only used in multi-viewports mode.
  init_info.SwapchainComposition = SDL_GPU_SWAPCHAINCOMPOSITION_SDR;  // Only used in multi-viewports mode.
  init_info.PresentMode = SDL_GPU_PRESENTMODE_VSYNC;
  cImGui_ImplSDLGPU3_Init(&init_info);
  
  return SDL_APP_CONTINUE;
}
