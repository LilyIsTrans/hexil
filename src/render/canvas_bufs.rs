use vkbuf::BufferContents;
use vulkano::buffer as vkbuf;
use vulkano::pipeline::graphics::vertex_input::Vertex;
#[derive(Debug, Clone, Copy, BufferContents, Vertex)]
#[repr(C)]
pub struct Position {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

pub const SQUARE: [Position; 5] = [
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
];

pub const SQUARE_IDX: [u16; 6] = [0, 1, 2, 3, 4, 1];

pub const HEXAGON: [Position; 7] = [
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
];
pub const HEXAGON_IDX: [u16; 8] = [0, 1, 2, 3, 4, 5, 6, 1];
