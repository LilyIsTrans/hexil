#include <SDL3/SDL.h>
#include <SDL3/SDL_init.h>
#include "hexil.h"

SDL_AppResult SDL_AppEvent(struct GlobalState *appstate, SDL_Event *event) {
  return SDL_APP_CONTINUE;
}
