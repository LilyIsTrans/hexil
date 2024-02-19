#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vk::buffer as vbuf;
use vulkano as vk;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// The type of grid associated with a project
pub enum GridType {
    Square,
    Hexagonal,
}

#[derive(Debug, Hash, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, Serialize, Deserialize)]
#[repr(C)]
pub struct CanvasSize {
    pub width: u64,
    pub height: u64,
}

impl CanvasSize {
    #[inline(always)]
    pub const fn area(&self) -> u64 {
        self.width * self.height
    }
}

/// Contains the state of a single open project. Starting with 1.0, Project types must never be removed, so that legacy projects can be opened and converted by any future Hexil.
pub struct ProjectV1 {
    /// The name of the project. Not necessarily the filename.
    name: String,
    /// The size of the canvas in grid tiles
    size: CanvasSize,
    /// The layers of the project
    layers: Vec<LayerV1>,
    /// Which grid type the project uses
    gridtype: GridType,
}
pub(crate) type Color = palette::Oklab;
pub(crate) type Palette = Vec<Color>;

pub(crate) type CanvasIndices = Vec<u32>;

#[derive(Debug, Serialize, Deserialize)]
/// It is fundamentally impossible to deserialize a data structure involving subbuffers with serde. This type exists as a quick and easy go between, so that we can still save projects anyway.
pub enum LayerV1Canvas {
    Alpha(Vec<f32>),
    BaseColor {
        palette: parking_lot::RwLock<Palette>,
        canvas: parking_lot::RwLock<CanvasIndices>,
    },
    Shading(Vec<i32>),
}

pub mod transfer_canvas_to_device;
/// A layer for a `ProjectV1`
pub struct LayerV1 {
    /// An optional user-defined name for the layer
    name: Option<String>,
    /// Size of the canvas. Currently must be identical to project canvas size.
    size: CanvasSize,
    /// The associated canvas
    canvas: LayerV1Canvas,
}
