#include <SDL3/SDL.h>
#include <SDL3/SDL_init.h>

void set_metadata() {
  SDL_SetAppMetadataProperty(SDL_PROP_APP_METADATA_NAME_STRING, "Hexil");
  SDL_SetAppMetadataProperty(SDL_PROP_APP_METADATA_VERSION_STRING, HEXIL_VERSION);
  SDL_SetAppMetadataProperty(SDL_PROP_APP_METADATA_IDENTIFIER_STRING, "ca.lilymccabe.hexil");
  SDL_SetAppMetadataProperty(SDL_PROP_APP_METADATA_CREATOR_STRING, "Lily Marie McCabe");
  SDL_SetAppMetadataProperty(SDL_PROP_APP_METADATA_COPYRIGHT_STRING, "Copyright (c) 2025 Lily Marie McCabe");
  SDL_SetAppMetadataProperty(SDL_PROP_APP_METADATA_URL_STRING, "https://github.com/LilyIsTrans/hexil");
  SDL_SetAppMetadataProperty(SDL_PROP_APP_METADATA_TYPE_STRING, "application");
}
