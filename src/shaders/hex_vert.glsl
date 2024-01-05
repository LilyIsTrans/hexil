#version 460

layout(location = 0) in vec2 position;

// layout( binding = 0 ) readonly buffer Project
// {
    // uvec2 dimensions;
    // vec2 top_left;
    // vec2 extent;
    // vec3 col[];
// } ProjectData;
// layout(binding = 1) readonly buffer Canvas {
//     uint indices[];
// } CanvasIndices;

layout(location = 1) out vec3 color;

/// Ported from `palette` crate
vec3 oklab_to_linear_srgb(vec3 c)
{
    // float l_ = c.x +  0.3963377774 * c.y +  0.2158037573 * c.z;
    
    // float m_ = c.x + -0.1055613458 * c.y + -0.0638541728 * c.z;
    // float s_ = c.x + -0.0894841775 * c.y + -1.2914855480 * c.z;

    vec3 lms_ = mat3(
        1.0000000000,  1.0000000000,  1.0000000000,
        0.3963377774, -0.1055613458, -0.0894841775,
        0.2158037573, -0.0638541728, -1.2914855480
    ) * c;

    // float l = l_ * l_ * l_;
    // float m = m_ * m_ * m_;
    // float s = s_ * s_ * s_;

    vec3 lms = lms_ * lms_ * lms_;

    // return mat3(
    //      4.0767416621 * l + -3.3077115913 * m +  0.2309699292 * s,
    //     -1.2684380046 * l +  2.6097574011 * m + -0.3413193965 * s,
    //     -0.0041960863 * l + -0.7034186147 * m +  1.7076147010 * s,
    // ) * lms;
    return mat3(
         4.0767416621, -1.2684380046, -0.0041960863,
        -3.3077115913,  2.6097574011, -0.7034186147,
         0.2309699292, -0.7034186147,  1.7076147010
    ) * lms;
}

float rand(vec2 co){
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}


struct ProjDat {
    uvec2 dimensions;
} ProjectData;
 

void main() {

    ProjectData.dimensions = uvec2(2, 2);
    vec3 cols[4] = vec3[4](
            vec3(1., 1., 1.),
            vec3(0., 0., 0.),
            vec3(0.7, 0., 1.),
            vec3(0.1, 0.8, 1.)
        );
    
    ivec2 coordinate = ivec2(gl_InstanceIndex % ProjectData.dimensions.x, gl_InstanceIndex / ProjectData.dimensions.x);
    // uint index = CanvasIndices.indices[gl_InstanceIndex];


    // color = oklab_to_linear_srgb(ProjectData.col[index]);

    color = cols[coordinate.x + coordinate.y * ProjectData.dimensions.x];
    
    // Step 1: Fit the entire canvas into clip space (from [-1, -1] to [1, 1])
    
    // vec2 tile_size = vec2(2.f) / vec2(ProjectData.dimensions);
    vec2 pos = position * 0.1;
    // pos += (vec2(coordinate) - (vec2(ProjectData.dimensions) / 2.f)) * tile_size;
    pos += (vec2(coordinate) - 1.f) / 5.f;
    
    
    // Step 2: Scale up by the dimensions of the grid divided by the dimensions of the viewport.
    // This zooms in to the appropriate scale.
    
    

    
    gl_Position = vec4(position * 0.8, 0.0, 0.0);

}
