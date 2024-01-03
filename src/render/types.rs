use bytemuck::{Pod, TransparentWrapper, Zeroable};
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(Debug, Clone, Copy, Hash, Zeroable, TransparentWrapper, Pod, Vertex)]
#[repr(transparent)]
pub struct PaletteIndex {
    #[format(R32_UINT)]
    pub idx: u32,
}
#[derive(Debug, Clone, Copy, Zeroable, Pod, Vertex)]
#[repr(C)]
pub struct Position {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
}
#[derive(Debug, Clone, Copy, Zeroable, TransparentWrapper, Pod, Vertex)]
#[repr(transparent)]
pub struct ColorOklab {
    #[format(R32G32B32_SFLOAT)]
    pub col: palette::Oklab,
}
