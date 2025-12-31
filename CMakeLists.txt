CMAKE_MINIMUM_REQUIRED(VERSION 4.2)

PROJECT(Hexil VERSION 0.1.0)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_EXTENSIONS ON)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/$<CONFIGURATION>")
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/$<CONFIGURATION>")

find_package(Vulkan REQUIRED)

add_custom_command(
  OUTPUT venv
  COMMAND python
  ARGS -m venv ${CMAKE_BINARY_DIR}/venv
)
set_source_files_properties(venv SYMBOLIC True)

add_custom_command(
  OUTPUT ${CMAKE_BINARY_DIR}/include/dcimgui.h ${CMAKE_BINARY_DIR}/dcimgui.cpp
  BYPRODUCTS ${CMAKE_BINARY_DIR}/dcimgui.json
  DEPENDS venv
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/vendored/dear_bindings/
  COMMAND ${CMAKE_BINARY_DIR}/venv/bin/pip
  ARGS install -r requirements.txt
  COMMAND ${CMAKE_BINARY_DIR}/venv/bin/python
  ARGS dear_bindings.py -o ${CMAKE_BINARY_DIR}/dcimgui ${CMAKE_SOURCE_DIR}/vendored/imgui/imgui.h
  COMMAND cmake
  ARGS -E make_directory ${CMAKE_BINARY_DIR}/include
  COMMAND cmake
  ARGS -E rename ${CMAKE_BINARY_DIR}/dcimgui.h ${CMAKE_BINARY_DIR}/include/dcimgui.h
)
add_custom_command(
  OUTPUT ${CMAKE_BINARY_DIR}/include/dcimgui_impl_sdl3.h ${CMAKE_BINARY_DIR}/dcimgui_impl_sdl3.cpp
  BYPRODUCTS ${CMAKE_BINARY_DIR}/dcimgui_impl_sdl3.json
  DEPENDS venv
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/vendored/dear_bindings/
  COMMAND ${CMAKE_BINARY_DIR}/venv/bin/pip
  ARGS install -r requirements.txt
  COMMAND ${CMAKE_BINARY_DIR}/venv/bin/python
  ARGS dear_bindings.py --include ${CMAKE_SOURCE_DIR}/vendored/imgui/imgui.h --backend -o ${CMAKE_BINARY_DIR}/dcimgui_impl_sdl3 ${CMAKE_SOURCE_DIR}/vendored/imgui/backends/imgui_impl_sdl3.h
  COMMAND cmake
  ARGS -E make_directory ${CMAKE_BINARY_DIR}/include
  COMMAND cmake
  ARGS -E rename ${CMAKE_BINARY_DIR}/dcimgui_impl_sdl3.h ${CMAKE_BINARY_DIR}/include/dcimgui_impl_sdl3.h
)
add_custom_command(
  OUTPUT ${CMAKE_BINARY_DIR}/include/dcimgui_impl_vulkan.h ${CMAKE_BINARY_DIR}/dcimgui_impl_vulkan.cpp
  BYPRODUCTS ${CMAKE_BINARY_DIR}/dcimgui_impl_vulkan.json
  DEPENDS venv
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/vendored/dear_bindings/
  COMMAND ${CMAKE_BINARY_DIR}/venv/bin/pip
  ARGS install -r requirements.txt
  COMMAND ${CMAKE_BINARY_DIR}/venv/bin/python
  ARGS dear_bindings.py --include ${CMAKE_SOURCE_DIR}/vendored/imgui/imgui.h --backend -o ${CMAKE_BINARY_DIR}/dcimgui_impl_vulkan ${CMAKE_SOURCE_DIR}/vendored/imgui/backends/imgui_impl_vulkan.h
  COMMAND cmake
  ARGS -E make_directory ${CMAKE_BINARY_DIR}/include
  COMMAND cmake
  ARGS -E rename ${CMAKE_BINARY_DIR}/dcimgui_impl_vulkan.h ${CMAKE_BINARY_DIR}/include/dcimgui_impl_vulkan.h
)
add_custom_command(
  OUTPUT ${CMAKE_BINARY_DIR}/include/imconfig.h
  DEPENDS ${CMAKE_SOURCE_DIR}/vendored/imgui/imconfig.h
  COMMAND cmake
  ARGS -E make_directory ${CMAKE_BINARY_DIR}/include
  COMMAND cmake
  ARGS -E copy -t ${CMAKE_BINARY_DIR}/include ${CMAKE_SOURCE_DIR}/vendored/imgui/imconfig.h
)

file(GLOB ImGuiSOURCES ${CMAKE_SOURCE_DIR}/vendored/imgui/*.cpp)

add_library(dcimgui STATIC
  ${CMAKE_BINARY_DIR}/dcimgui.cpp 
  ${CMAKE_BINARY_DIR}/dcimgui_impl_sdl3.cpp 
  ${CMAKE_BINARY_DIR}/dcimgui_impl_vulkan.cpp
  ${ImGuiSOURCES}
  ${CMAKE_SOURCE_DIR}/vendored/imgui/backends/imgui_impl_sdl3.cpp 
  ${CMAKE_SOURCE_DIR}/vendored/imgui/backends/imgui_impl_vulkan.cpp
)

target_sources(dcimgui PUBLIC ${CMAKE_BINARY_DIR}/include/imconfig.h)
target_include_directories(dcimgui PUBLIC ${CMAKE_BINARY_DIR}/include)
target_include_directories(dcimgui PRIVATE ${CMAKE_SOURCE_DIR}/vendored/imgui ${CMAKE_SOURCE_DIR}/vendored/imgui/backends)
ADD_SUBDIRECTORY(./src ./obj)



