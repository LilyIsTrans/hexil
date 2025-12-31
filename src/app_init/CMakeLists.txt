add_library(app_init STATIC app_init.c set_app_metadata.c)
target_link_libraries(hexil app_init)
target_link_libraries(app_init dcimgui)
target_include_directories(app_init PUBLIC ${Vulkan_INCLUDE_DIRS})
