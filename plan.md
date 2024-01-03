# Pipline:
CPU >>==<uniform vec2 canvas_dim>==<uniform [col_ok] palette>==<vertex_buffer [u32 as point_primitive] tiles>==>> Geometry Shader
Geometry Shader >>==<palette[tiles.idx] oklab -> srgb -> col -> 6 triangle_primitives <- position canvas_dim & tiles.idx>==>> Vertex Shader
Vertex Shader >>==<clip space position in whole canvas -> screen space coordinate (potentially outside viewport if zoomed in)>==>> Fragment Shader
Fragment Shader >>==<Default. Maybe antialiasing on hexagon edges? That's more tesselation's job though. >==>> Window!
