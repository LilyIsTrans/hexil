
#version 460

layout(location = 1) in vec3 color;
layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(color, 1.0);
}
