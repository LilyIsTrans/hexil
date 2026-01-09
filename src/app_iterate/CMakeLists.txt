add_library(app_iterate STATIC app_iterate.c)
target_link_libraries(hexil app_iterate volk::volk)
target_link_libraries(app_iterate dcimgui)
target_include_directories(app_iterate PRIVATE ${CMAKE_SOURCE_DIR}/src/app_init)
