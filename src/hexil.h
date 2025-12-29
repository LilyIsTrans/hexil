#include <SDL3/SDL.h>
#include <SDL3/SDL_gpu.h>


struct HexilGlobalState  {
  SDL_Window* main_window;
  SDL_GPUDevice* gpu_device;
};
