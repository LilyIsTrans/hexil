add_library(app_event STATIC app_event.c)
target_link_libraries(hexil app_event volk::volk)
target_link_libraries(app_event dcimgui)
