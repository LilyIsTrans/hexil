#include <SDL3/SDL.h>
#include <SDL3/SDL_error.h>
#include <SDL3/SDL_init.h>
#include <SDL3/SDL_log.h>
#include <SDL3/SDL_messagebox.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "hexil.h"
#include "set_app_metadata.h"

SDL_AppResult SDL_AppInit(struct GlobalState **appstate, int argc, char **argv) {
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
  return SDL_APP_SUCCESS;
}
