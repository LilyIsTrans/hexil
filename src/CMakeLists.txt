# Create an option to switch between a system sdl library and a vendored SDL library
option(USE_VENDORED_SDL "Use vendored SDL3" OFF)

if(USE_VENDORED_SDL )
    # This assumes you have added SDL as a submodule in vendored/SDL
    # TODO: Implement
    add_subdirectory(vendored/SDL EXCLUDE_FROM_ALL)
else()
    # 1. Look for a SDL3 package,
    # 2. look for the SDL3-shared component, and
    # 3. fail if the shared component cannot be found.
    find_package(SDL3 REQUIRED CONFIG REQUIRED COMPONENTS SDL3-shared)
endif()


add_executable(hexil WIN32 hexil.c)

target_link_libraries(hexil SDL3::SDL3)
