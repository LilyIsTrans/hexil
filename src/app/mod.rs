#![allow(dead_code)]
use std::sync::Arc;

/// The current latest project type.
pub type Project = ProjectV1;

/// Contains the state of a single instance of Hexil. It probably doesn't make sense to ever have more than one of these.
pub struct AppInstance {
    /// The open tabs. For now, should always have exactly 1 element.
    tabs: Arc<Vec<Project>>,
    /// Channel to send commands to the renderer
    render_channel: std::sync::mpsc::Sender<crate::render::RenderCommand>,
    /// Event Loop Proxy to send commands to the windower
    window_channel: winit::event_loop::EventLoopProxy<crate::window::WindowCommand>,
    /// Handle to the render thread
    render_thread: std::thread::JoinHandle<Result<(), crate::render::RendererError>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
/// A canvas for a `ProjectV1` layer
pub enum LayerV1Canvas {
    /// A canvas of alpha blending values.
    Alpha(Arc<[f64]>),
    /// A canvas of base colours, with an associated palette.
    BaseColour {
        palette: Arc<Vec<palette::Oklab>>,
        canvas: Arc<[usize]>,
    },
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
