use std::sync::Arc;

use super::renderer_error;
use tracing::instrument;
use try_log::log_tries;

use vulkano as vk;

use super::Renderer;

impl Renderer {
    /// Makes a new `Renderer`.
    #[instrument(skip_all)]
    #[log_tries(tracing::error)]
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

        Ok(Self {
            instance,
            window,
            surface,
            physical_device,
            logical_device,
            command_allocator,
            graphics_queue,
            transfer_queue,
            allocator,
        })
    }
}
