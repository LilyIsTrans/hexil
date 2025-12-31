add_library(app_iterate STATIC app_iterate.c)
target_link_libraries(hexil app_iterate volk::volk)
target_link_libraries(app_iterate dcimgui)
