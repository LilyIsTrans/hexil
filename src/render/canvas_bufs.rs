use vkbuf::BufferContents;
use vulkano::buffer as vkbuf;
use vulkano::pipeline::graphics::vertex_input::Vertex;

/// Represents a position in 2D space. Typically, in Hexil, once a coordinate is in a `Position`, it is taken to be in Normalized Device Coordinates.
#[derive(Debug, Clone, Copy, BufferContents, Vertex)]
#[repr(C)]
pub struct Position {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

/// Holds the vertices needed to define a square that fills Normalized Device Coordinate space using TRIANGLE_FAN geometry.
pub const SQUARE: [Position; 6] = [
    Position {
        position: [0.0f32, 0.0f32],
    },
    Position {
        position: [-1.0f32, -1.0f32],
    },
    Position {
        position: [1.0f32, -1.0f32],
    },
    Position {
        position: [-1.0f32, 1.0f32],
    },
    Position {
        position: [1.0f32, 1.0f32],
    },
    Position {
        position: [-1.0f32, -1.0f32],
    },
];

/// Holds the vertices needed to define a hexagon that fills Normalized Device Coordinate space using TRIANGLE_FAN geometry.
pub const HEXAGON: [Position; 8] = [
    Position {
        position: [0.0f32, 0.0f32],
    },
    Position {
        position: [-0.5f32, -1.0f32],
    },
    Position {
        position: [0.5f32, -1.0f32],
    },
    Position {
        position: [1.0f32, 0.0f32],
    },
    Position {
        position: [0.5f32, 1.0f32],
    },
    Position {
        position: [-0.5f32, 1.0f32],
    },
    Position {
        position: [-1.0f32, 0.0f32],
    },
    Position {
        position: [-0.5f32, -1.0f32],
    },
];
