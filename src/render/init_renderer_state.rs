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

        let physical_device = Self::get_physical_device(instance.clone(), surface.as_ref())?;

        let (logical_device, transfer_queue, graphics_queue) =
            Self::get_queues_and_device(physical_device.clone())?;

        let allocator = Arc::new(vk::memory::allocator::StandardMemoryAllocator::new_default(
            logical_device.clone(),
        ));

        let command_allocator = vk::command_buffer::allocator::StandardCommandBufferAllocator::new(
            logical_device.clone(),
            vk::command_buffer::allocator::StandardCommandBufferAllocatorCreateInfo::default(),
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
