use std::sync::{mpsc::RecvError, Arc};
use tracing::{error, info_span, instrument};

use thiserror::Error;
use tracing::info;
use vk::{swapchain::Surface, VulkanLibrary};
use vulkano as vk;
use winit::window::Window;

mod canvas_bufs;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("{0}")]
    VkLoadErr(vk::LoadingError),
    #[error("{0}")]
    VkErr(vk::VulkanError),
    #[error("{0}")]
    ValidVkErr(vk::Validated<vk::VulkanError>),
    #[error("{0}")]
    WindowHandleError(winit::raw_window_handle::HandleError),
    #[error("No physical devices? At all!? Seriously, as far as this program can tell, you must be reading this through a serial port, which like, props, but what on earth made you think a pixel art program would work with that?")]
    NoPhysicalDevices,
    #[error("{0}")]
    ChannelError(RecvError),
}

impl From<RecvError> for RendererError {
    fn from(v: RecvError) -> Self {
        Self::ChannelError(v)
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
    queues: Arc<[Arc<vk::device::Queue>]>,
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
    let renderer = Renderer::initialize(window);
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
        match render_command_channel.recv() {
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
    pub fn initialize(window: Arc<winit::window::Window>) -> Result<Self, RendererError> {
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

        let queue_create_info = vk::device::QueueCreateInfo {
            flags: vk::device::QueueCreateFlags::empty(),
            ..Default::default()
        };

        let logical_device = vk::device::DeviceCreateInfo {
            queue_create_infos: vec![queue_create_info],
            enabled_extensions: physical_device.supported_extensions().intersection(
                &vk::device::DeviceExtensions {
                    khr_16bit_storage: true,
                    khr_8bit_storage: true,
                    khr_acceleration_structure: true,
                    khr_bind_memory2: true,
                    khr_buffer_device_address: true,
                    khr_copy_commands2: true,
                    khr_create_renderpass2: true,
                    khr_dedicated_allocation: true,
                    khr_deferred_host_operations: true,
                    khr_depth_stencil_resolve: true,
                    khr_descriptor_update_template: true,
                    khr_device_group: true,
                    khr_display_swapchain: true,
                    khr_draw_indirect_count: true,
                    khr_driver_properties: true,
                    khr_dynamic_rendering: true,
                    khr_external_fence: true,
                    khr_external_fence_fd: true,
                    khr_external_fence_win32: true,
                    khr_external_memory: true,
                    khr_external_memory_fd: true,
                    khr_external_memory_win32: true,
                    khr_external_semaphore: true,
                    khr_external_semaphore_fd: true,
                    khr_external_semaphore_win32: true,
                    khr_format_feature_flags2: true,
                    khr_fragment_shader_barycentric: true,
                    khr_fragment_shading_rate: true,
                    khr_get_memory_requirements2: true,
                    khr_global_priority: true,
                    khr_image_format_list: true,
                    khr_imageless_framebuffer: true,
                    khr_incremental_present: true,
                    khr_maintenance1: true,
                    khr_maintenance2: true,
                    khr_maintenance3: true,
                    khr_maintenance4: true,
                    khr_map_memory2: true,
                    khr_multiview: true,
                    khr_performance_query: true,
                    khr_pipeline_executable_properties: true,
                    khr_pipeline_library: true,
                    khr_portability_subset: true,
                    khr_present_id: true,
                    khr_present_wait: true,
                    khr_push_descriptor: true,
                    khr_relaxed_block_layout: true,
                    khr_sampler_mirror_clamp_to_edge: true,
                    khr_sampler_ycbcr_conversion: true,
                    khr_separate_depth_stencil_layouts: true,
                    khr_shader_atomic_int64: true,
                    khr_shader_clock: true,
                    khr_shader_draw_parameters: true,
                    khr_shader_float16_int8: true,
                    khr_shader_float_controls: true,
                    khr_shader_integer_dot_product: true,
                    khr_shader_non_semantic_info: true,
                    khr_shader_subgroup_extended_types: true,
                    khr_shader_subgroup_uniform_control_flow: true,
                    khr_shader_terminate_invocation: true,
                    khr_shared_presentable_image: true,
                    khr_spirv_1_4: true,
                    khr_storage_buffer_storage_class: true,
                    khr_swapchain: true,
                    khr_swapchain_mutable_format: true,
                    khr_synchronization2: true,
                    khr_timeline_semaphore: true,
                    khr_uniform_buffer_standard_layout: true,
                    khr_variable_pointers: true,
                    khr_vulkan_memory_model: true,
                    khr_win32_keyed_mutex: true,
                    khr_workgroup_memory_explicit_layout: true,
                    khr_zero_initialize_workgroup_memory: true,
                    ..Default::default()
                },
            ),
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

        Ok(Self {
            instance,
            window,
            surface,
            physical_device,
            logical_device,
            queues,
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
}
