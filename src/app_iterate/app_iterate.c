#include "dcimgui.h"
#include "dcimgui_impl_sdl3.h"
#include "dcimgui_impl_vulkan.h"
#include "hexil.h"
#include <SDL3/SDL.h>
#include <SDL3/SDL_gpu.h>
#include <SDL3/SDL_init.h>
SDL_AppResult SDL_AppIterate(struct HexilGlobalState *appstate) {
  // (After event loop)
  // Start the Dear ImGui frame
  cImGui_ImplVulkan_NewFrame();
  cImGui_ImplSDL3_NewFrame();
  ImGui_NewFrame();
  ImGui_DockSpaceOverViewport();
  bool _true = true;
  ImGui_ShowDemoWindow(&_true); // Show demo window! :)

  ImGui_Render();
  ImDrawData *draw_data = ImGui_GetDrawData();
  const bool is_minimized =
      (draw_data->DisplaySize.x <= 0.0f || draw_data->DisplaySize.y <= 0.0f);
  SDL_GPUCommandBuffer *render_buffer =
      SDL_AcquireGPUCommandBuffer(appstate->gpu_device); // Acquire a GPU command buffer
  SDL_GPUCommandBuffer *copy_buffer =
      SDL_AcquireGPUCommandBuffer(appstate->gpu_device); // Acquire a GPU command buffer

  SDL_GPUTexture *swapchain_texture;
  SDL_WaitAndAcquireGPUSwapchainTexture(render_buffer, appstate->main_window,
                                        &swapchain_texture, NULL,
                                        NULL); // Acquire a swapchain texture

  if (swapchain_texture != NULL && !is_minimized) {
    // This is mandatory: call ImGui_ImplSDLGPU3_PrepareDrawData() to upload the
    // vertex/index buffer!
    

    // Setup and start a render pass
    SDL_GPUColorTargetInfo target_info;
    memset(&target_info, 0, sizeof(target_info));
    target_info.texture = swapchain_texture;
    SDL_FColor sdl_clear_color = {0.0f, 0.0f, 0.0f, 0.0f};
    target_info.clear_color = sdl_clear_color;
    target_info.load_op = SDL_GPU_LOADOP_CLEAR;
    target_info.store_op = SDL_GPU_STOREOP_STORE;
    target_info.mip_level = 0;
    target_info.layer_or_depth_plane = 0;
    target_info.cycle = false;
    SDL_GPURenderPass *render_pass =
        SDL_BeginGPURenderPass(render_buffer, &target_info, 1, NULL);

    // Render ImGui
    cImGui_ImplVulkan_RenderDrawData(draw_data, render_buffer);

    SDL_EndGPURenderPass(render_pass);
  }
  SDL_SubmitGPUCommandBuffer(render_buffer);
  return SDL_APP_CONTINUE;
}
