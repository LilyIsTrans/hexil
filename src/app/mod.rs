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

#[derive(Debug, Clone)]
/// A canvas for a `ProjectV1` layer
/// It is an invariant of this type that all the subbuffers are HOST_RANDOM_ACCESS, and are never written by the device. If you don't know what that means, don't worry about it.
pub enum LayerV1Canvas {
    /// A canvas of alpha blending values.
    Alpha(vbuf::Subbuffer<[f32]>),
    /// A canvas of base colours, with an associated palette.
    BaseColor {
        palette: vbuf::Subbuffer<[palette::Oklab]>,
        canvas: vbuf::Subbuffer<[u32]>,
    },
    /// A canvas of shading steps (positive: brighten, negative: darken)
    Shading(vbuf::Subbuffer<[i32]>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// It is fundamentally impossible to deserialize a data structure involving subbuffers with serde. This type exists as a quick and easy go between, so that we can still save projects anyway.
pub enum LayerV1CanvasHost {
    Alpha(Vec<f32>),
    BaseColor {
        palette: Vec<palette::Oklab>,
        canvas: Vec<u32>,
    },
    Shading(Vec<i32>),
}

impl TryFrom<LayerV1Canvas> for LayerV1CanvasHost {
    type Error = vk::sync::HostAccessError;

    fn try_from(
        value: LayerV1Canvas,
    ) -> std::result::Result<LayerV1CanvasHost, vk::sync::HostAccessError> {
        Ok(match value {
            LayerV1Canvas::Alpha(buf) => Self::Alpha(buf.read()?.to_vec()),
            LayerV1Canvas::BaseColor { palette, canvas } => Self::BaseColor {
                palette: palette.read()?.to_vec(),
                canvas: canvas.read()?.to_vec(),
            },
            LayerV1Canvas::Shading(buf) => Self::Shading(buf.read()?.to_vec()),
        })
    }
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
