add_library(app_quit STATIC app_quit.c)
target_link_libraries(hexil app_quit volk::volk)
target_link_libraries(app_quit dcimgui)
