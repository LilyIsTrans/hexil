use std::{
    sync::{
        mpsc::{RecvError, RecvTimeoutError, TryRecvError},
        Arc,
    },
    time::Duration,
};
use tracing::{error, instrument, warn};

use thiserror::Error;
use tracing::info;
use vk::{
    command_buffer::PrimaryCommandBufferAbstract, memory::allocator::MemoryAllocator,
    sync::GpuFuture, VulkanLibrary,
};
use vulkano as vk;
use winit::window::Window;

mod canvas_bufs;
mod consts;
use crate::render::canvas_bufs::{Position, HEXAGON, HEXAGON_IDX, SQUARE, SQUARE_IDX};

use self::consts::*;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("{0}")]
    VkLoadErr(vk::LoadingError),
    #[error("{0}")]
    VkErr(vk::VulkanError),
    #[error("{0}")]
    ValidVkErr(vk::Validated<vk::VulkanError>),
    #[error("{0}")]
    ValidBufErr(vk::Validated<vk::buffer::AllocateBufferError>),
    #[error("{0}")]
    WindowHandleError(winit::raw_window_handle::HandleError),
    #[error("No physical devices? At all!? Seriously, as far as this program can tell, you must be reading this through a serial port, which like, props, but what on earth made you think a pixel art program would work with that?")]
    NoPhysicalDevices,
    #[error("{0}")]
    ChannelError(RecvTimeoutError),
    #[error("No graphics queues available!")]
    NoGraphicsQueues,
    #[error("No transfer queues available!")]
    NoTransferQueues,
    #[error("{0}")]
    VkValidationErr(Box<vk::ValidationError>),
    #[error("{0}")]
    VkCommandBufExecErr(vk::command_buffer::CommandBufferExecError),
}

impl From<RecvTimeoutError> for RendererError {
    fn from(v: RecvTimeoutError) -> Self {
        Self::ChannelError(v)
    }
}

impl From<vk::command_buffer::CommandBufferExecError> for RendererError {
    fn from(v: vk::command_buffer::CommandBufferExecError) -> Self {
        Self::VkCommandBufExecErr(v)
    }
}

impl From<Box<vk::ValidationError>> for RendererError {
    fn from(v: Box<vk::ValidationError>) -> Self {
        Self::VkValidationErr(v)
    }
}

impl From<vk::Validated<vk::buffer::AllocateBufferError>> for RendererError {
    fn from(v: vk::Validated<vk::buffer::AllocateBufferError>) -> Self {
        Self::ValidBufErr(v)
    }
}

impl From<winit::raw_window_handle::HandleError> for RendererError {
    fn from(v: winit::raw_window_handle::HandleError) -> Self {
        Self::WindowHandleError(v)
    }
}

impl From<vk::Validated<vk::VulkanError>> for RendererError {
    fn from(v: vk::Validated<vk::VulkanError>) -> Self {
        Self::ValidVkErr(v)
    }
}

impl From<vk::VulkanError> for RendererError {
    fn from(v: vk::VulkanError) -> Self {
        Self::VkErr(v)
    }
}

impl From<vk::LoadingError> for RendererError {
    fn from(v: vk::LoadingError) -> Self {
        Self::VkLoadErr(v)
    }
}

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderCommand {
    /// The renderer must handle the window being resized to the new dimensions (in physical pixels).
    WindowResized([u32; 2]),
    /// The renderer should shut down gracefully
    Shutdown,
}

#[instrument]
pub fn render_thread(
    window: Arc<Window>,
    render_command_channel: std::sync::mpsc::Receiver<RenderCommand>,
) -> Result<(), RendererError> {
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
    #[instrument]
    pub fn new(window: Arc<winit::window::Window>) -> Result<Self, RendererError> {
        let lib = VulkanLibrary::new()?;

        tracing::info!(
            "Successfully loaded Vulkan version {}.{}.{}",
            lib.api_version().major,
            lib.api_version().minor,
            lib.api_version().patch
        );

        let ext_span = tracing::info_span!("Vulkan Extensions:");
        {
            let _guard = ext_span.entered();
            for prop in lib.extension_properties() {
                tracing::info!(
                    "Extension support detected: {} version {}",
                    prop.extension_name,
                    prop.spec_version
                );
            }
        }
        let mut needs_wayland_surface: bool = false;
        let mut needs_win32_surface: bool = false;
        let mut needs_xcb_surface: bool = false;
        let mut needs_xlib_surface: bool = false;

        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        match window.window_handle()?.as_raw() {
            RawWindowHandle::Xlib(_) => needs_xlib_surface = true,
            RawWindowHandle::Xcb(_) => needs_xcb_surface = true,
            RawWindowHandle::Wayland(_) => needs_wayland_surface = true,
            RawWindowHandle::Win32(_) => needs_win32_surface = true,
            _ => error!("Unsupported windowing system!"),
        };

        let needed_extensions = vk::instance::InstanceExtensions {
            khr_surface: true,
            khr_wayland_surface: needs_wayland_surface,
            khr_win32_surface: needs_win32_surface,
            khr_xcb_surface: needs_xcb_surface,
            khr_xlib_surface: needs_xlib_surface,
            khr_get_surface_capabilities2: true,
            ..Default::default()
        };

        let mut info = vk::instance::InstanceCreateInfo::application_from_cargo_toml();
        info.enabled_extensions = needed_extensions;

        info!("Attempting to create instance...");

        let instance = vk::instance::Instance::new(lib, info)?;
        info!("Instance created successfully!");

        let surface = vk::swapchain::Surface::from_window(instance.clone(), window.clone());

        if let Err(surface_error) = surface.clone() {
            match surface_error {
                vk::Validated::Error(e) => error!("Surface Creation Error: {e}"),
                vk::Validated::ValidationError(e) => {
                    error!("Surface Creation Validation Error: {e}")
                }
            };
        };
        let surface = surface?;

        let physical_device = instance
            .enumerate_physical_devices()?
            .inspect(|dev| info!("Physical Device detected: {}", dev.properties().device_name))
            .max_by_key(|dev| dev.properties().max_instance_count) // TODO: Decide which device to use by a more sophisticated method than simply whichever can have the most instances
            .ok_or(RendererError::NoPhysicalDevices)?;

        info!(
            "Selected Physical Device: {}",
            physical_device.properties().device_name
        );

        let graphics_queue_flags = vk::device::QueueFlags::GRAPHICS;
        let transfer_queue_flags = vk::device::QueueFlags::TRANSFER;
        let both = graphics_queue_flags.union(transfer_queue_flags);
        let queue_family_indices = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .filter(|(_, p)| p.queue_flags.intersects(both));
        let graphics_queue_families: Vec<usize> = queue_family_indices
            .clone()
            .filter(|(_, p)| p.queue_flags.contains(graphics_queue_flags))
            .map(|(i, _)| i)
            .collect();
        let transfer_queue_families: Vec<usize> = queue_family_indices
            .filter(|(_, p)| p.queue_flags.contains(transfer_queue_flags))
            .map(|(i, _)| i)
            .collect();

        let (both, graphics_only): (Vec<usize>, Vec<usize>) = graphics_queue_families
            .iter()
            .partition(|i| transfer_queue_families.contains(i));
        let transfer_only: Vec<usize> = transfer_queue_families
            .iter()
            .filter(|i| !graphics_only.contains(i))
            .copied()
            .collect();
        // Selects a graphics queue family and a transfer queue family.
        // If possible, it will select different queue families.
        let (graphics_family, transfer_family) =
            match (both.len(), graphics_only.len(), transfer_only.len()) {
                (0, 0, _) => {
                    error!("No graphics queues!");
                    return Err(RendererError::NoGraphicsQueues);
                }
                (0, _, 0) => {
                    error!("No transfer queues!");
                    return Err(RendererError::NoTransferQueues);
                }
                (1, 0, 0) => {
                    warn!("Only one queue available, performance may be affected.");
                    let q = both
                        .first()
                        .expect("We just confirmed that both has exactly 1 element.");
                    (*q, *q)
                }
                (_, 0, 0) => (both[0], both[1]),
                (_, 0, _) => (both[0], transfer_only[0]),
                (_, _, 0) => (graphics_only[0], both[0]),
                (_, _, _) => (graphics_only[0], transfer_only[0]),
            };

        let mut queues = vec![0.5];
        if graphics_family == transfer_family {
            queues.push(0.5);
        }
        let graphics_queue_create_info = vk::device::QueueCreateInfo {
            queue_family_index: graphics_family
                .try_into()
                .expect("I got this index from this device. It better be able to take it back."),
            queues,
            ..Default::default()
        };

        let transfer_queue_create_info = vk::device::QueueCreateInfo {
            queue_family_index: transfer_family
                .try_into()
                .expect("I got this index from this device. It better be able to take it back."),
            ..Default::default()
        };

        let mut queue_create_infos = Vec::<vk::device::QueueCreateInfo>::with_capacity(2);

        queue_create_infos.push(graphics_queue_create_info);
        if graphics_family != transfer_family {
            queue_create_infos.push(transfer_queue_create_info);
        };

        let logical_device = vk::device::DeviceCreateInfo {
            queue_create_infos,
            enabled_extensions: physical_device
                .supported_extensions()
                .intersection(&ALL_KHR_DEVICE_EXTENSIONS),
            // enabled_features: Features {
            //     triangle_fans: true,
            //     ..Default::default()
            // },
            ..Default::default()
        };

        let device_create = vk::device::Device::new(physical_device.clone(), logical_device);

        let (logical_device, queues) = match device_create {
            Err(device_error) => {
                match device_error.clone() {
                    vk::Validated::Error(e) => error!("Device Creation Error: {e}"),
                    vk::Validated::ValidationError(e) => {
                        error!("Device Creation Validation Error: {e}")
                    }
                };
                Err(device_error)
            }
            Ok(dev) => Ok(dev),
        }?;

        let queues = queues.collect::<Arc<[_]>>();
        let graphics_queue: Arc<vk::device::Queue> = queues
            .iter()
            .find(|q| {
                graphics_family
                    == q.queue_family_index()
                        .try_into()
                        .expect("I sure hope u32 fits into usize.")
            })
            .expect("If it didn't exist, we'd have returned an error a few lines ago.")
            .clone();
        let transfer_queue: Arc<vk::device::Queue> = queues
            .iter()
            .find(|q| {
                transfer_family
                    == q.queue_family_index()
                        .try_into()
                        .expect("I sure hope u32 fits into usize.")
                    && q.id_within_family() != graphics_queue.id_within_family()
            })
            .expect("If it didn't exist, we'd have returned an error a few lines ago.")
            .clone();

        let immutable_allocator = Arc::new(
            vk::memory::allocator::StandardMemoryAllocator::new_default(logical_device.clone()),
        );

        let square_stage_buffer =
            Self::make_staging_buffer(immutable_allocator.clone(), SQUARE.iter().copied())?;
        let hex_stage_buffer =
            Self::make_staging_buffer(immutable_allocator.clone(), HEXAGON.iter().copied())?;
        let square_idx_stage_buffer =
            Self::make_staging_buffer(immutable_allocator.clone(), SQUARE_IDX.iter().copied())?;
        let hex_idx_stage_buffer =
            Self::make_staging_buffer(immutable_allocator.clone(), HEXAGON_IDX.iter().copied())?;

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
        let square_idx_buffer: vk::buffer::Subbuffer<[u16]> = Self::make_buffer(
            immutable_allocator.clone(),
            vk::buffer::BufferUsage::TRANSFER_DST | vk::buffer::BufferUsage::INDEX_BUFFER,
            vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
            std::mem::size_of_val(&SQUARE_IDX),
            SQUARE_IDX.len(),
        )?;
        let hex_idx_buffer: vk::buffer::Subbuffer<[u16]> = Self::make_buffer(
            immutable_allocator.clone(),
            vk::buffer::BufferUsage::TRANSFER_DST | vk::buffer::BufferUsage::INDEX_BUFFER,
            vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
            std::mem::size_of_val(&HEXAGON_IDX),
            HEXAGON_IDX.len(),
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
            square_buffer,
        ))?;
        transfer_builder.copy_buffer(vk::command_buffer::CopyBufferInfoTyped::buffers(
            square_idx_stage_buffer,
            square_idx_buffer,
        ))?;
        transfer_builder.copy_buffer(vk::command_buffer::CopyBufferInfoTyped::buffers(
            hex_stage_buffer,
            hex_buffer,
        ))?;
        transfer_builder.copy_buffer(vk::command_buffer::CopyBufferInfoTyped::buffers(
            hex_idx_stage_buffer,
            hex_idx_buffer,
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
        })
    }

    fn make_swapchain(&mut self, new_size: [u32; 2]) -> Result<(), RendererError> {
        if new_size == [0u32, 0u32] {
            self.swapchain = None;
        } else if self
            .swapchain
            .as_ref()
            .is_some_and(|swapchain| swapchain.0.image_extent() == new_size)
        {
        } else if let Some(swapchain) = self.swapchain.clone() {
            let mut create_info = swapchain.0.create_info();
            create_info.image_extent = new_size;
            self.swapchain = Some(swapchain.0.recreate(create_info)?);
        } else {
            let swapchain = vk::swapchain::SwapchainCreateInfo {
                image_format: Default::default(),
                image_view_formats: Default::default(),
                image_extent: new_size,
                image_usage: vk::image::ImageUsage::STORAGE, // TODO: Might need to be updated to allow for displaying
                pre_transform: vk::swapchain::SurfaceTransform::Identity,
                composite_alpha: vk::swapchain::CompositeAlpha::Opaque,
                present_mode: vk::swapchain::PresentMode::Fifo,
                // present_modes: todo!(), // TODO: Add support for changing this in a settings menu
                ..Default::default()
            };
            let swapchain = vk::swapchain::Swapchain::new(
                self.logical_device.clone(),
                self.surface.clone(),
                swapchain,
            )?;
            self.swapchain = Some(swapchain);
        }
        Ok(())
    }

    fn make_buffer<T: vk::buffer::BufferContents>(
        allocator: Arc<impl vk::memory::allocator::MemoryAllocator>,
        usage: vk::buffer::BufferUsage,
        memory_type: vk::memory::allocator::MemoryTypeFilter,
        size: usize,
        len: usize,
    ) -> Result<vk::buffer::Subbuffer<[T]>, RendererError> {
        let raw_square_buf_info = vk::buffer::BufferCreateInfo {
            flags: vk::buffer::BufferCreateFlags::empty(),
            size: (size)
                .try_into()
                .expect("I sure hope the size of a few floats and u16s fits in a u64"),
            usage,
            ..Default::default()
        };
        let mut square_buf_info = raw_square_buf_info.clone();
        square_buf_info.size = 0;
        let square_buf_info = square_buf_info;
        let square_alloc_info = vk::memory::allocator::AllocationCreateInfo {
            memory_type_filter: memory_type,
            ..Default::default()
        };
        let square_buf = vk::buffer::Buffer::new_unsized(
            allocator.clone(),
            square_buf_info,
            square_alloc_info,
            len.try_into().unwrap(),
        );
        Ok(square_buf?)
    }

    fn make_staging_buffer<T: vk::buffer::BufferContents>(
        allocator: Arc<impl vk::memory::allocator::MemoryAllocator>,
        iter: impl Iterator<Item = T> + std::iter::ExactSizeIterator,
    ) -> Result<vk::buffer::Subbuffer<[T]>, RendererError> {
        Self::make_buffer(
            allocator,
            vk::buffer::BufferUsage::TRANSFER_SRC,
            vk::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            std::mem::size_of::<T>() * iter.len(),
            iter.len(),
        )
    }
}

mod pipeline;
