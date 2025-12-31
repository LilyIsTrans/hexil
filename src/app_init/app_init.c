#include "dcimgui.h"
#include "dcimgui_impl_sdl3.h"
#include "dcimgui_impl_vulkan.h"
#include "hexil.h"
#include "set_app_metadata.h"
#include <SDL3/SDL.h>
#include <SDL3/SDL_assert.h>
#include <SDL3/SDL_error.h>
#include <SDL3/SDL_hints.h>
#include <SDL3/SDL_init.h>
#include <SDL3/SDL_log.h>
#include <SDL3/SDL_messagebox.h>
#include <SDL3/SDL_video.h>
#include <SDL3/SDL_vulkan.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <vulkan/vk_platform.h>
#include <vulkan/vulkan_core.h>

void hexil_create_global_vulkan_instance(struct HexilGlobalState *appstate) {
  uint32_t instance_extension_count;
  const char *const *instance_extensions =
      SDL_Vulkan_GetInstanceExtensions(&instance_extension_count);

  if (SDL_GetLogPriority(SDL_LOG_CATEGORY_GPU) >= SDL_LOG_PRIORITY_INFO) {
    SDL_LogInfo(SDL_LOG_CATEGORY_GPU,
                "SDL Reported the following %" PRIu32
                " Vulkan instance extensions are required:\n",
                instance_extension_count);
    for (uint32_t i = 0; i < instance_extension_count; ++i) {
      SDL_LogInfo(SDL_LOG_CATEGORY_GPU, "\t%s\n", instance_extensions[i]);
    }
  }

  VkInstanceCreateInfo instance_create_info = {
      .sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
      .pNext = NULL, // TODO (Maybe): Implement support for non-vulkan
                     // platforms by attaching a translation-layer driver in a
                     // VkDirectDriverLoadingListLUNARG here
      .flags = 0,
      .pApplicationInfo = NULL,
      .enabledLayerCount = 0,
      .ppEnabledLayerNames = NULL,
      .enabledExtensionCount = instance_extension_count,
      .ppEnabledExtensionNames = instance_extensions};

  SDL_assert_always(vkCreateInstance(&instance_create_info, NULL,
                                     &appstate->vulkan_state.instance) ==
                    VK_SUCCESS);
}

bool hexil_init_SDL() {
#ifdef SDL_PLATFORM_LINUX
  SDL_SetHint(SDL_HINT_VIDEO_DRIVER, "wayland,x11");
#endif
  if (!SDL_Init(SDL_INIT_VIDEO)) {
    const char *const err = SDL_GetError();

    char manual_format_string[] =
        "Failed to initialize SDL! That's basically unrecoverable for a poor "
        "helpless automated program such as myself, so I'm probably going to "
        "close in a moment. SDL says the reason for the failure is: ";

    char *error_message =
        malloc(strlen(err) + sizeof(manual_format_string) + 1);

    strcpy(error_message, manual_format_string);
    strcat(error_message, err);
    strcat(error_message, "\n");

    SDL_LogError(SDL_LOG_CATEGORY_APPLICATION, "%s", error_message);

    if (!SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR,
                                  "Uh-oh! Failed to initialize SDL!",
                                  error_message, NULL)) {
      SDL_LogError(
          SDL_LOG_CATEGORY_APPLICATION,
          "Somewhat unsurprisingly, after failing to init, SDL also failed to "
          "spawn an error message window! In case it's at all helpful, the "
          "error message from SDL for that new failure is: %s\n",
          SDL_GetError());
    }

    return false;
  }
  return true;
}

void hexil_init_global_state(struct HexilGlobalState **appstate) {
  *appstate = malloc(sizeof(struct HexilGlobalState));
  memset(*appstate, 0, sizeof(struct HexilGlobalState));
}

HexilOsWindow hexil_create_new_os_window(struct HexilGlobalState *appstate,
                                         const char *const window_title,
                                         int width, int height) {
  HexilOsWindow output;

  SDL_WindowFlags window_flags =
      SDL_WINDOW_RESIZABLE | SDL_WINDOW_HIGH_PIXEL_DENSITY | SDL_WINDOW_VULKAN;
  output.window = SDL_CreateWindow(window_title, width, height, window_flags);

  SDL_assert_always(SDL_Vulkan_CreateSurface(
      output.window, appstate->vulkan_state.instance, NULL, &output.surface));

  return output;
}

void hexil_init_imgui(struct HexilGlobalState* appstate)
{  // Setup Dear ImGui context
  CIMGUI_CHECKVERSION();
  ImGui_CreateContext(NULL);
  ImGuiIO *io = ImGui_GetIO();
  io->ConfigFlags |=
      ImGuiConfigFlags_NavEnableKeyboard; // Enable Keyboard Controls
  io->ConfigFlags |=
      ImGuiConfigFlags_NavEnableGamepad; // Enable Gamepad Controls
  io->ConfigFlags |= ImGuiConfigFlags_DockingEnable;

  // Setup Platform/Renderer backends
  cImGui_ImplSDL3_InitForVulkan(appstate->main_window.window);
  ImGui_ImplVulkan_InitInfo init_info = {
    
  };
  cImGui_ImplVulkan_Init(&init_info);
}
SDL_AppResult SDL_AppInit(struct HexilGlobalState **appstate, int argc,
                          char **argv) {
  hexil_set_SDL_metadata();
  if (!hexil_init_SDL()) {
    return SDL_APP_FAILURE;
  }

  hexil_init_global_state(appstate);

  (*appstate)->main_window =
      hexil_create_new_os_window(*appstate, "Hexil", 720, 480);

  // TODO: Init Vulkan
  
  hexil_init_imgui(*appstate);

  return SDL_APP_CONTINUE;
}
