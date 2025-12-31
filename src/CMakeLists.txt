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



add_executable(hexil WIN32 hexil.c hexil.h)

target_link_libraries(hexil SDL3::SDL3)
target_link_libraries(hexil dcimgui)
target_link_libraries(hexil ${Vulkan_LIBRARIES})

target_include_directories(hexil PUBLIC ${Vulkan_INCLUDE_DIRS})


add_compile_definitions(HEXIL_VERSION="${Hexil_VERSION}" HEXIL_VERSION_MAJOR="${Hexil_VERSION_MAJOR}" HEXIL_VERSION_MINOR="${Hexil_VERSION_MINOR}" HEXIL_VERSION_PATCH="${Hexil_VERSION_PATCH}" HEXIL_VERSION_TWEAK="${Hexil_VERSION_TWEAK}")
include_directories(.)


add_subdirectory(app_init ./app_init)
add_subdirectory(app_iterate ./app_iterate)
add_subdirectory(app_event ./app_event)
add_subdirectory(app_quit ./app_quit)

