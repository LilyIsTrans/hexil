use std::sync::Arc;

use super::{renderer_error, types::Position};
use tracing::instrument;
use vk::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    image::view::ImageView,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    render_pass::{Framebuffer, FramebufferCreateInfo},
};
use vulkano as vk;

use super::Renderer;

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

        let allocator = Arc::new(vk::memory::allocator::StandardMemoryAllocator::new_default(
            logical_device.clone(),
        ));

        let command_allocator = vk::command_buffer::allocator::StandardCommandBufferAllocator::new(
            logical_device.clone(),
            vk::command_buffer::allocator::StandardCommandBufferAllocatorCreateInfo::default(),
        );

        let canvas_size_buf = vk::buffer::Buffer::from_data(
            allocator.clone(),
            vk::buffer::BufferCreateInfo {
                usage: vk::buffer::BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            vk::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vk::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            crate::app::CanvasSize {
                width: 128u64,
                height: 128u64,
            },
        );

        let vertex1 = Position {
            position: [-0.5, -0.5],
        };
        let vertex2 = Position {
            position: [0.0, 0.5],
        };
        let vertex3 = Position {
            position: [0.5, -0.25],
        };

        let vertex_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vec![vertex1, vertex2, vertex3],
        )?;

        Ok(Self {
            instance,
            window,
            surface,
            physical_device,
            logical_device,
            command_allocator,
            graphics_queue,
            transfer_queue,
            render_pass: None,
            framebuffers: None,
            vertex_buffer,
            swapchain: None,
            allocator,
        })
    }
}
