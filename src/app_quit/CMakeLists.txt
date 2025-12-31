add_library(app_quit STATIC app_quit.c)
target_link_libraries(hexil app_quit)
target_link_libraries(app_quit dcimgui)
target_include_directories(app_quit PUBLIC ${Vulkan_INCLUDE_DIRS})
