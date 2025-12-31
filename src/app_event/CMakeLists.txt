add_library(app_event STATIC app_event.c)
target_link_libraries(hexil app_event)
target_link_libraries(app_event dcimgui)
target_include_directories(app_event PUBLIC ${Vulkan_INCLUDE_DIRS})
