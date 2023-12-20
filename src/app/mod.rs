use std::sync::Arc;

use crate::render::RenderCommand;
use crate::window::WindowCommand;
use parking_lot::RwLock;
use vulkano::buffer as buf;
use vulkano::shader as sha;
use winit::dpi::LogicalPosition;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;

/// Contains the state of a single instance of Hexil. It probably doesn't make sense to ever have more than one of these.
pub struct AppInstance {}

/// The type of grid associated with a project
pub enum GridType {
    Square,
    Hexagonal,
}

/// Contains the state of a single open project. Starting with 1.0, Project types must never be removed, so that legacy projects can be opened and converted by any future Hexil.
pub struct ProjectV1 {
    /// The name of the project. Not necessarily the filename.
    name: String,
    /// The width of the project canvas, in grid cells
    width: usize,
    /// The height of the project canvas, in grid cells
    height: usize,
    /// The layers of the project
    layers: Arc<Vec<LayerV1>>,
    /// Which grid type the project uses
    gridtype: GridType,
}

/// A canvas for a `ProjectV1` layer
pub enum LayerV1Canvas {
    /// A canvas of base colours, with an associated palette.
    BaseColour {
        palette: Arc<Vec<palette::Oklab>>,
        canvas: Arc<[usize]>,
    },
    /// A canvas of alpha blending values.
    Alpha(Arc<[f64]>),
    /// A canvas of shading steps (positive: brighten, negative: darken)
    Shading(Arc<[i32]>),
}

/// A layer for a `ProjectV1`
pub struct LayerV1 {
    /// An optional user-defined name for the layer
    name: Option<String>,
    /// The width of the layer canvas, in grid cells. Currently, this must be identical to the Project canvas `width`.
    width: usize,
    /// The height of the project canvas, in grid cells. Currently, this must be identical to the Project canvas `height`.
    height: usize,
    /// The associated canvas
    canvas: LayerV1Canvas,
}
