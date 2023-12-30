mod instance_create;
mod select_physical_device;

mod queue_device_creation;

mod make_swapchain;

mod buffer_make;
mod lib_select;
mod pipeline;
mod surface_create;
use std::{sync::Arc, time::Duration};
use tracing::{error, instrument};

use tracing::info;
use vk::{buffer::Subbuffer, command_buffer::PrimaryCommandBufferAbstract, sync::GpuFuture};
use vulkano as vk;
use winit::window::Window;

mod canvas_bufs;
mod consts;
use crate::render::canvas_bufs::{Position, HEXAGON, SQUARE};

mod renderer_error;
pub use renderer_error::*;

/// Holds the entire state of Hexil's rendering system. It probably doesn't make sense to ever make more than one of this, but technically nothing is stopping you.
#[allow(dead_code)]
pub struct Renderer {
    instance: Arc<vk::instance::Instance>,
    window: Arc<winit::window::Window>,
    surface: Arc<vk::swapchain::Surface>,
    physical_device: Arc<vk::device::physical::PhysicalDevice>,
    logical_device: Arc<vk::device::Device>,
    command_allocator: vk::command_buffer::allocator::StandardCommandBufferAllocator,
    graphics_queue: Arc<vk::device::Queue>,
    transfer_queue: Arc<vk::device::Queue>,
    swapchain: Option<(Arc<vk::swapchain::Swapchain>, Vec<Arc<vk::image::Image>>)>,
    hex_buf: Subbuffer<[Position]>,
    square_buf: Subbuffer<[Position]>,
}

/// A command that can be sent to the main render thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderCommand {
    /// The renderer must handle the window being resized to the new dimensions (in physical pixels).
    WindowResized([u32; 2]),
    /// The renderer should shut down gracefully
    Shutdown,
}

/// Runs Hexil's rendering system. Should be run in it's own dedicated OS thread.
#[instrument]
pub fn render_thread(
    window: Arc<Window>,
    render_command_channel: std::sync::mpsc::Receiver<RenderCommand>,
) -> Result<(), renderer_error::RendererError> {
    let renderer = Renderer::new(window);
    if let Err(e) = renderer {
        error!("Failed to init renderer! {}", e);
        return Err(e);
    }
    let mut renderer = renderer?;

    for dev in renderer.instance.enumerate_physical_devices()? {
        info!(
            "Found physical device: \"{:#?}\"",
            dev.properties().device_name
        );
    }

    renderer.window.set_visible(true);
    loop {
        match render_command_channel.recv_timeout(Duration::from_nanos(100)) {
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),
            Err(e) => {
                error!("Inter-thread communication failure! {}", e);
                return Err(e.into());
            }
            Ok(RenderCommand::WindowResized(new_size)) => renderer.make_swapchain(new_size)?,
            Ok(RenderCommand::Shutdown) => return Ok(()),
        }
    }
}

impl Renderer {
    /// Makes a new `Renderer`.
    #[instrument]
    pub fn new(window: Arc<winit::window::Window>) -> Result<Self, renderer_error::RendererError> {
        let lib = Self::get_vulkan_library()?;

        let instance = Self::get_instance(lib, window.clone())?;

        let surface = Self::get_surface(instance.clone(), window.clone())?;

        let physical_device = Self::get_physical_device(instance.clone())?;

        let (logical_device, transfer_queue, graphics_queue) =
            Self::get_queues_and_device(physical_device.clone())?;

        let immutable_allocator = Arc::new(
            vk::memory::allocator::StandardMemoryAllocator::new_default(logical_device.clone()),
        );

        let square_stage_buffer =
            Self::make_staging_buffer(immutable_allocator.clone(), SQUARE.iter().copied())?;
        let hex_stage_buffer =
            Self::make_staging_buffer(immutable_allocator.clone(), HEXAGON.iter().copied())?;

        let square_buffer: vk::buffer::Subbuffer<[Position]> = Self::make_buffer(
            immutable_allocator.clone(),
            vk::buffer::BufferUsage::TRANSFER_DST | vk::buffer::BufferUsage::VERTEX_BUFFER,
            vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
            std::mem::size_of_val(&SQUARE),
            SQUARE.len(),
        )?;
        let hex_buffer: vk::buffer::Subbuffer<[Position]> = Self::make_buffer(
            immutable_allocator.clone(),
            vk::buffer::BufferUsage::TRANSFER_DST | vk::buffer::BufferUsage::VERTEX_BUFFER,
            vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
            std::mem::size_of_val(&HEXAGON),
            HEXAGON.len(),
        )?;

        let command_allocator = vk::command_buffer::allocator::StandardCommandBufferAllocator::new(
            logical_device.clone(),
            vk::command_buffer::allocator::StandardCommandBufferAllocatorCreateInfo::default(),
        );

        let mut transfer_builder = vk::command_buffer::AutoCommandBufferBuilder::primary(
            &command_allocator,
            transfer_queue.queue_family_index(),
            vk::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )?;
        transfer_builder.copy_buffer(vk::command_buffer::CopyBufferInfoTyped::buffers(
            square_stage_buffer,
            square_buffer.clone(),
        ))?;
        transfer_builder.copy_buffer(vk::command_buffer::CopyBufferInfoTyped::buffers(
            hex_stage_buffer,
            hex_buffer.clone(),
        ))?;
        let transfer_command = transfer_builder.build()?;
        transfer_command.execute(transfer_queue.clone())?.flush()?;

        Ok(Self {
            instance,
            window,
            surface,
            physical_device,
            logical_device,
            command_allocator,
            graphics_queue,
            transfer_queue,
            swapchain: None,
            hex_buf: hex_buffer,
            square_buf: square_buffer,
        })
    }
}
