add_library(app_init STATIC app_init.c set_app_metadata.c init_vulkan.c)
target_link_libraries(hexil app_init volk::volk)
target_link_libraries(app_init dcimgui)
