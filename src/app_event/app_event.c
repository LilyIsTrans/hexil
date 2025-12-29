#include <SDL3/SDL.h>
#include <SDL3/SDL_events.h>
#include <SDL3/SDL_init.h>
#include "hexil.h"
#include "dcimgui.h"
#include "dcimgui_impl_sdl3.h"

SDL_AppResult SDL_AppEvent(struct HexilGlobalState *appstate, SDL_Event *event) {
  // (Where your code calls SDL_PollEvent())
  cImGui_ImplSDL3_ProcessEvent(event); // Forward your event to backend
  // (You should discard mouse/keyboard messages in your game/engine when io.WantCaptureMouse/io.WantCaptureKeyboard are set.)
  switch (event->type) {
    case SDL_EVENT_QUIT:
      return SDL_APP_SUCCESS;
    default:
      break;
  }
  return SDL_APP_CONTINUE;
}
