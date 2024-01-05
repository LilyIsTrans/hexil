#version 460

layout(location = 0) in vec2 position;
layout(location = 1) out vec3 color;

const uint WIDTH = 20;
const uint HEIGHT = 15;

vec2 transform_one(vec2 initial) {
    vec2 pos = initial / (vec2(0.75 * WIDTH, HEIGHT));
    vec2 offset = (vec2(0.75) / (vec2(0.75 * WIDTH, HEIGHT)))  + vec2((2.f * vec2((gl_InstanceIndex % WIDTH), (gl_InstanceIndex / WIDTH) + (gl_InstanceIndex % 2) / 2.f )) / vec2(WIDTH, HEIGHT)) - 1.f;
    return pos + offset;
}


void main() {
    vec2 offsets[2] = vec2[2](
            vec2(0.3, 0.3),
            -vec2(0.3, 0.3)
        );

    vec3 colors[6] = vec3[6](
            vec3(1.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            vec3(1.0, 0.0, 1.0),
            vec3(1.0, 1.0, 0.0),
            vec3(0.0, 1.0, 1.0)
        );
    
    vec2 pos = transform_one(position);
    
    gl_Position = vec4(pos, 0.0, 1.0);
    color = colors[gl_InstanceIndex % 6];
}
