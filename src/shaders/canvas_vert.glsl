#version 460

layout(location = 0) in vec2 position;
layout(location = 1) out vec3 color;

layout(set = 0, binding = 0) readonly buffer CanvasSettings {
    uint WIDTH;
    uint HEIGHT;
    vec3 palette[];
} Settings;
layout(set = 0, binding = 1) readonly buffer CanvasIndices {
    uint indices[];
} Indices;




vec2 transform_one(vec2 initial) {
    vec2 scaled_pos = initial / vec2(Settings.WIDTH * 0.75, Settings.HEIGHT);
    float column_offset = ((gl_InstanceIndex % 2) / 2.f);
    vec2 grid_offset = 2.f * (vec2(gl_InstanceIndex % Settings.WIDTH, (gl_InstanceIndex / Settings.WIDTH + column_offset)) + 0.5f);
    vec2 top_left = vec2(-1.f);
    vec2 canvas_size = vec2(Settings.WIDTH, Settings.HEIGHT);
    vec2 offset = top_left + (grid_offset / canvas_size);
    
    return scaled_pos + offset;
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
