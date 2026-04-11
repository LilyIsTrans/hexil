use std::{backtrace::Backtrace, mem::MaybeUninit, num::NonZeroU32, sync::atomic::AtomicU64};

use anyhow::anyhow;
use palette::stimulus::IntoStimulus;
use pkg_version::{pkg_version_major, pkg_version_minor, pkg_version_patch};
use raw_window_handle::RawWindowHandle;
use tracing::{debug, error, info, warn};
use vulkanalia::vk::SwapchainCreateInfoKHRBuilder;

use crate::hexil_prelude::all::*;

pub struct GlobalState {
    pub vulkan_state: VulkanState,
    pub window_state: WindowState,
}

pub struct PresentID {
    inner: AtomicU64,
}

impl PresentID {
    pub const ZERO: Self = Self {
        inner: AtomicU64::new(0),
    };

    pub fn next_present_id(&self) -> u64 {
        self.inner
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
        // This will always return a monotonically increasing, nonzero id
        // as long as we never present more than 18446744073709551615 times
        // to the same swapchain. Assuming an average framerate of 65537
        // frames per second, this would mean the application ran continuously
        // without even invalidating the swapchain for exactly 281470681808895
        // seconds, or just over 8.9 million years.
        // I therefore consider it reasonable to declare as a constraint of Hexil;
        // each window must be resized at least once every 8 million years,
        // (the extra ~0.9 million years and the fact that most users do not
        // have displays which run at 65537Hz can be used to account
        // for timer imprecision if neccessary), or the program may exhibit undefined
        // behaviour.
    }
}

pub struct Surface {
    pub surface: vk::SurfaceKHR,
    pub swapchain: Option<Swapchain>,
    pub swapchain_format: vk::Format,
    pub supported_swapchain_view_formats: Vec<vk::Format>,
    pub selected_color_space: vk::ColorSpaceKHR,
    pub selected_alpha_composite_mode: vk::CompositeAlphaFlagsKHR,
    pub selected_present_mode: vk::PresentModeKHR,
    pub min_image_count: u32,
}

pub struct Swapchain {
    pub swapchain: vk::SwapchainKHR,
    pub last_present_id: PresentID,
}

pub struct HexilWindow {
    pub window: winit::window::Window,
    pub surface: Option<Surface>,
}

pub struct WindowState {
    pub main_window: HexilWindow,
}

fn format_color_depth(format: &vk::Format) -> Option<u32> {
    match *format {
        vk::Format::R16G16B16_UNORM => Some(16),
        vk::Format::R16G16B16_SNORM => Some(16),
        vk::Format::R16G16B16A16_UNORM => Some(16),
        vk::Format::R16G16B16A16_SNORM => Some(16),

        vk::Format::A2R10G10B10_UNORM_PACK32 => Some(10),
        vk::Format::A2R10G10B10_SNORM_PACK32 => Some(10),

        vk::Format::A2B10G10R10_UNORM_PACK32 => Some(10),
        vk::Format::A2B10G10R10_SNORM_PACK32 => Some(10),

        vk::Format::R8G8B8_UNORM => Some(8),
        vk::Format::R8G8B8_SNORM => Some(8),
        vk::Format::B8G8R8_UNORM => Some(8),
        vk::Format::B8G8R8_SNORM => Some(8),

        vk::Format::A8B8G8R8_UNORM_PACK32 => Some(8),
        vk::Format::A8B8G8R8_SNORM_PACK32 => Some(8),

        vk::Format::R8G8B8A8_UNORM => Some(8),
        vk::Format::R8G8B8A8_SNORM => Some(8),
        vk::Format::B8G8R8A8_UNORM => Some(8),
        vk::Format::B8G8R8A8_SNORM => Some(8),

        _ => None,
    }
}

fn score_formats_for_underlying_swapchain_image(format: &vk::Format) -> u32 {
    match *format {
        vk::Format::R16G16B16_UNORM => 0,
        vk::Format::R16G16B16_SNORM => 1,
        vk::Format::R16G16B16A16_UNORM => 2,
        vk::Format::R16G16B16A16_SNORM => 3,

        vk::Format::A2R10G10B10_UNORM_PACK32 => 4,
        vk::Format::A2R10G10B10_SNORM_PACK32 => 5,

        vk::Format::A2B10G10R10_UNORM_PACK32 => 6,
        vk::Format::A2B10G10R10_SNORM_PACK32 => 7,

        vk::Format::R8G8B8_UNORM => 10,
        vk::Format::R8G8B8_SNORM => 10,
        vk::Format::B8G8R8_UNORM => 10,
        vk::Format::B8G8R8_SNORM => 10,

        vk::Format::A8B8G8R8_UNORM_PACK32 => 15,
        vk::Format::A8B8G8R8_SNORM_PACK32 => 16,

        vk::Format::R8G8B8A8_UNORM => 17,
        vk::Format::R8G8B8A8_SNORM => 18,
        vk::Format::B8G8R8A8_UNORM => 17,
        vk::Format::B8G8R8A8_SNORM => 18,

        _ => u32::MAX,
    }
}

fn score_color_spaces(space: &vk::ColorSpaceKHR) -> u32 {
    match *space {
        vk::ColorSpaceKHR::DCI_P3_NONLINEAR_EXT => 0,
        vk::ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT => 1,
        vk::ColorSpaceKHR::DISPLAY_P3_LINEAR_EXT => 2,
        vk::ColorSpaceKHR::HDR10_ST2084_EXT => 3,
        vk::ColorSpaceKHR::BT2020_LINEAR_EXT => 4,
        vk::ColorSpaceKHR::HDR10_HLG_EXT => 5,
        vk::ColorSpaceKHR::ADOBERGB_NONLINEAR_EXT => 6,
        vk::ColorSpaceKHR::ADOBERGB_LINEAR_EXT => 7,

        vk::ColorSpaceKHR::SRGB_NONLINEAR => 8,

        _ => u32::MAX,
    }
}
fn is_color_space_hdr(space: &vk::ColorSpaceKHR) -> bool {
    match *space {
        vk::ColorSpaceKHR::DCI_P3_NONLINEAR_EXT
        | vk::ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT
        | vk::ColorSpaceKHR::DISPLAY_P3_LINEAR_EXT
        | vk::ColorSpaceKHR::HDR10_ST2084_EXT
        | vk::ColorSpaceKHR::BT2020_LINEAR_EXT
        | vk::ColorSpaceKHR::HDR10_HLG_EXT => true,

        _ => false,
    }
}
fn score_present_modes(mode: &vk::PresentModeKHR) -> u32 {
    match *mode {
        vk::PresentModeKHR::MAILBOX => 0,
        vk::PresentModeKHR::FIFO_LATEST_READY => 1,
        vk::PresentModeKHR::FIFO => 2,

        _ => u32::MAX,
    }
}
fn compare_surface_formats(
    format: &vk::SurfaceFormat2KHR,
    other: &vk::SurfaceFormat2KHR,
) -> std::cmp::Ordering {
    let color_depth_comparison = format_color_depth(&format.surface_format.format)
        .unwrap_or(0)
        .cmp(&format_color_depth(&other.surface_format.format).unwrap_or(0));
    let overall_format_comparison =
        score_formats_for_underlying_swapchain_image(&format.surface_format.format).cmp(
            &score_formats_for_underlying_swapchain_image(&other.surface_format.format),
        );
    let color_space_comparison = score_color_spaces(&format.surface_format.color_space)
        .cmp(&score_color_spaces(&other.surface_format.color_space));
    compare_surface_formats_inner(
        color_depth_comparison,
        overall_format_comparison,
        color_space_comparison,
    )
}

fn compare_surface_formats_inner(
    color_depth_comparison: std::cmp::Ordering,
    overall_format_comparison: std::cmp::Ordering,
    color_space_comparison: std::cmp::Ordering,
) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    match (
        color_depth_comparison,
        overall_format_comparison,
        color_space_comparison,
    ) {
        (Less, Less, Less) => Less,
        (Less, Less, Equal) => Less,
        (Less, Less, Greater) => Less,
        (Less, Equal, Less) => Less,
        (Less, Equal, Equal) => Less,
        (Less, Equal, Greater) => Equal,
        (Less, Greater, Less) => Less,
        (Less, Greater, Equal) => Less,
        (Less, Greater, Greater) => Greater,
        (Equal, Less, Less) => Less,
        (Equal, Less, Equal) => Less,
        (Equal, Less, Greater) => Greater,
        (Equal, Equal, Less) => Less,
        (Equal, Equal, Equal) => Equal,
        (Equal, Equal, Greater) => Greater,
        (Equal, Greater, Less) => Less,
        (Equal, Greater, Equal) => Greater,
        (Equal, Greater, Greater) => Greater,
        (Greater, Less, Less) => Less,
        (Greater, Less, Equal) => Greater,
        (Greater, Less, Greater) => Greater,
        (Greater, Equal, Less) => Equal,
        (Greater, Equal, Equal) => Greater,
        (Greater, Equal, Greater) => Greater,
        (Greater, Greater, Less) => Greater,
        (Greater, Greater, Equal) => Greater,
        (Greater, Greater, Greater) => Greater,
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;
    use std::cmp::Ordering::*;

    use crate::global_state::compare_surface_formats_inner;

    fn ordering_strat() -> impl Strategy<Value = std::cmp::Ordering> {
        prop_oneof![
            // For cases without data, `Just` is all you need
            Just(std::cmp::Ordering::Less),
            Just(std::cmp::Ordering::Equal),
            Just(std::cmp::Ordering::Greater),
        ]
    }

    proptest! {

    #[test]
    fn surface_format_comparison_sanity(a in ordering_strat(), b in ordering_strat(), c in ordering_strat()) {
        prop_assert_eq!(compare_surface_formats_inner(a, b, c), compare_surface_formats_inner(a.reverse(), b.reverse(), c.reverse()).reverse())
    }
    }
}

impl HexilWindow {
    const SHARING_MODE: vk::SharingMode = vk::SharingMode::EXCLUSIVE;

    /// Call this whenever the size of the window has changed (including from, but not to, nonexistence).
    pub fn rebuild_swapchain(
        &mut self,
        vulkan_state: &VulkanState,
        new_size: (NonZeroU32, NonZeroU32),
    ) -> Result<()> {
        let surface: &mut Surface = match &mut self.surface {
            Some(s) => s,
            None => {
                self.create_surface(vulkan_state)?;
                self.surface
                    .as_mut()
                    .expect("We literally just created the surface.")
            }
        };
        let info = vk::SwapchainCreateInfoKHR::builder()
            .flags(
                vk::SwapchainCreateFlagsKHR::DEFERRED_MEMORY_ALLOCATION
                    | vk::SwapchainCreateFlagsKHR::PRESENT_WAIT_2
                    | vk::SwapchainCreateFlagsKHR::PRESENT_ID_2,
            )
            .surface(surface.surface)
            .min_image_count(surface.min_image_count)
            .image_format(surface.swapchain_format)
            .image_color_space(surface.selected_color_space)
            .image_extent(
                vk::Extent2D::builder()
                    .width(new_size.0.get())
                    .height(new_size.1.get())
                    .build(),
            )
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(Self::SHARING_MODE)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .image_color_space(surface.selected_color_space)
            .composite_alpha(surface.selected_alpha_composite_mode)
            .present_mode(surface.selected_present_mode)
            .clipped(true)
            .old_swapchain(
                surface
                    .swapchain
                    .as_ref()
                    .map_or(vk::SwapchainKHR::null(), |s| s.swapchain),
            );

        // Safety: All fields in `info` are set to valid values
        let swapchain = unsafe { vulkan_state.device.create_swapchain_khr(&info, None) }?;

        let swapchain = Swapchain {
            swapchain,
            last_present_id: PresentID::ZERO,
        };

        surface.swapchain = Some(swapchain);

        Ok(())
    }

    pub fn create_surface(&mut self, vulkan_state: &VulkanState) -> Result<()> {
        use vulkanalia::window::create_surface;

        let surface =
            (unsafe { create_surface(&vulkan_state.instance, &self.window, &self.window) })?;

        let mut surface_present_scaling_caps = vk::SurfacePresentScalingCapabilitiesKHR::builder();

        let mut surface_present_mode_compat = vk::SurfacePresentModeCompatibilityKHR::builder();

        let mut surface_capabilities = vk::SurfaceCapabilities2KHR::builder()
            .push_next(&mut surface_present_mode_compat)
            .push_next(&mut surface_present_scaling_caps)
            .build();

        let mut surface_present_mode = vk::SurfacePresentModeKHR::builder();

        let surface_info = vk::PhysicalDeviceSurfaceInfo2KHR::builder()
            .surface(surface)
            .push_next(&mut surface_present_mode);

        unsafe {
            vulkan_state
                .instance
                .get_physical_device_surface_capabilities2_khr(
                    vulkan_state.device.physical_device(),
                    &surface_info,
                    &mut surface_capabilities,
                )
        }?;

        let mut surface_present_modes: Vec<vk::PresentModeKHR> = Vec::with_capacity(
            surface_present_mode_compat
                .present_mode_count
                .try_into()
                .unwrap(),
        );

        surface_present_mode_compat.present_modes = surface_present_modes
            .spare_capacity_mut()
            .as_mut_ptr()
            .cast_init();

        unsafe {
            vulkan_state
                .instance
                .get_physical_device_surface_capabilities2_khr(
                    vulkan_state.device.physical_device(),
                    &surface_info,
                    &mut surface_capabilities,
                )
        }?;

        unsafe {
            surface_present_modes.set_len(
                surface_present_mode_compat
                    .present_mode_count
                    .try_into()
                    .unwrap(),
            )
        };

        let selected_present_mode = surface_present_modes.into_iter();
        #[cfg(feature = "extra-log-statements")]
        let selected_present_mode =
            selected_present_mode.inspect(|mode| debug!("Present mode {:?} supported!", mode));
        let selected_present_mode = selected_present_mode
            .min_by_key(score_present_modes)
            .ok_or_else(|| anyhow!("Zero present modes supported by surface! What?"))?;
        if score_present_modes(&selected_present_mode) == u32::MAX {
            error!(
                "No favoured present mode supported! Everything *should* work fine with any format, but consider running `hexil --vulkan-diagnostics` and emailing the developer with the resulting file at `hexil@lilymccabe.ca`!! She'd be delighted to include first-class support for your system in the next minor release :3"
            );
        }

        let surface_formats = unsafe {
            vulkan_state
                .instance
                .get_physical_device_surface_formats2_khr(
                    vulkan_state.device.physical_device(),
                    &surface_info,
                )
        }?;

        let surface = Surface {
            surface,
            swapchain: None,
            swapchain_format: todo!(),
            supported_swapchain_view_formats: todo!(),
            selected_color_space: todo!(),
            selected_alpha_composite_mode: todo!(),
            selected_present_mode,
            min_image_count: todo!(),
        };

        Ok(())
    }

    fn choose_underlying_format(supported_formats: &[vk::Format]) -> vk::Format {
        todo!()
    }
}

impl WindowState {
    pub fn new(eloop: &winit::event_loop::ActiveEventLoop) -> Result<Self> {
        Ok(Self {
            main_window: HexilWindow {
                window: eloop.create_window(
                    winit::window::WindowAttributes::default()
                        .with_active(true)
                        .with_title("Hexil"),
                )?,
                surface: None,
            },
        })
    }
}

unsafe impl Send for ActiveVulkanState {}
impl !Sync for ActiveVulkanState {}

pub struct ActiveVulkanState {
    pub graphics_command_pool: vk::CommandPool,
    pub transfer_command_pool: vk::CommandPool,

    pub primary_graphics_command_buffer: vk::CommandBuffer,
    pub primary_transfer_command_buffer: vk::CommandBuffer,
}

#[derive(Clone, Copy)]
pub struct QueueWithInfo {
    pub queue: vk::Queue,
    pub queue_family_index: u32,
    pub queue_index: u32,
}

pub struct VulkanState {
    pub required_extensions: Vec<vk::Extension>,
    pub entry: vulkanalia::Entry,
    pub instance: vulkanalia::Instance,
    pub device: vulkanalia::Device,
    pub graphics_queue: QueueWithInfo,
    pub transfer_queue: QueueWithInfo,
    active_state: Option<ActiveVulkanState>,
    pub present_id: AtomicU64,
}

impl GlobalState {
    /// Initializes Hexil's global state
    ///
    /// This will initialize both the window system state [`window_state`] and the
    /// vulkan library/graphics card state. This includes creating the main window,
    /// though nothing will be drawn to it.
    pub fn new(eloop: &winit::event_loop::ActiveEventLoop) -> Result<Self> {
        use raw_window_handle::HasWindowHandle;
        let window_state = WindowState::new(eloop)?;

        let vulkan_state =
            VulkanState::new(&window_state.main_window.window.window_handle()?.as_raw())?;

        Ok(Self {
            vulkan_state,
            window_state,
        })
    }
}

impl Drop for VulkanState {
    fn drop(&mut self) {
        if let Some(active_state) = &mut self.active_state {
            unsafe {
                self.device
                    .destroy_command_pool(active_state.graphics_command_pool, None)
            };
            active_state.graphics_command_pool = vk::Handle::null();
            unsafe {
                self.device
                    .destroy_command_pool(active_state.transfer_command_pool, None)
            };
            active_state.transfer_command_pool = vk::Handle::null();
        }
        self.active_state = None;

        unsafe { self.device.destroy_device(None) };
        self.graphics_queue.queue = vk::Handle::null();
        self.transfer_queue.queue = vk::Handle::null();

        unsafe { self.instance.destroy_instance(None) };
    }
}

impl VulkanState {
    pub fn new(main_window: &RawWindowHandle) -> Result<Self> {
        let loader =
            unsafe { vulkanalia::loader::LibloadingLoader::new(vulkanalia::loader::LIBRARY) }?;
        let entry = unsafe { vulkanalia::Entry::new(loader) }.map_err(|b| anyhow!("{}", b))?;
        let app_info = vk::ApplicationInfo::builder()
            .application_version(vk::make_version(
                pkg_version_major!(),
                pkg_version_minor!(),
                pkg_version_patch!(),
            ))
            .api_version(vk::make_version(1, 4, 0))
            .application_name(b"Hexil");

        let required_extensions: Vec<vk::Extension> = REQUIRED_EXTENSIONS
            .iter()
            .chain([Self::extension_for_window(main_window)?].iter())
            .copied()
            .collect();

        let required_instance_extensions = required_extensions
            .iter()
            .filter_map(|ext| match ext.type_ {
                "instance" => Some(ext.name.as_ptr()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let info = vk::InstanceCreateInfo::builder()
            .enabled_extension_names(&required_instance_extensions)
            .flags(vk::InstanceCreateFlags::empty())
            .application_info(&app_info);
        const VALIDATION_LAYERS_NAME: &'static [*const std::ffi::c_char] =
            &[bytemuck::must_cast_slice::<_, std::ffi::c_char>(
                b"VK_LAYER_KHRONOS_validation".as_slice(),
            )
            .as_ptr()];
        #[cfg(debug_assertions)]
        let info = info.enabled_layer_names(&VALIDATION_LAYERS_NAME);

        let instance = unsafe { entry.create_instance(&info, None) }?;

        let device = Self::create_device(&instance, None)?;
        Ok(Self {
            required_extensions,
            entry,
            instance,
            device: device.0,
            graphics_queue: device.1,
            transfer_queue: device.2,
            active_state: None,
            present_id: 0.into(),
        })
    }

    pub fn get_active_state(&mut self) -> Option<&mut ActiveVulkanState> {
        self.active_state.as_mut()
    }

    pub fn activate(&mut self) -> Result<()> {
        let gfx_cmd_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(self.graphics_queue.queue_family_index);
        let xfer_cmd_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(self.transfer_queue.queue_family_index);

        let graphics_command_pool = unsafe {
            self.device
                .create_command_pool(&gfx_cmd_pool_create_info, None)
        }?;
        let transfer_command_pool = unsafe {
            self.device
                .create_command_pool(&xfer_cmd_pool_create_info, None)
        }?;

        let gfx_cmd_buf_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(graphics_command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);
        let xfer_cmd_buf_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(transfer_command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);

        let primary_graphics_command_buffer = unsafe {
            self.device
                .allocate_command_buffers(&gfx_cmd_buf_alloc_info)
        }?[0];
        let primary_transfer_command_buffer = unsafe {
            self.device
                .allocate_command_buffers(&xfer_cmd_buf_alloc_info)
        }?[0];

        if let Some(active_state) = &mut self.active_state {
            unsafe {
                self.device.device_wait_idle()?;
                self.device
                    .destroy_command_pool(active_state.graphics_command_pool, None);
                self.device
                    .destroy_command_pool(active_state.transfer_command_pool, None);
            }
        }

        self.active_state = Some(ActiveVulkanState {
            graphics_command_pool,
            transfer_command_pool,
            primary_graphics_command_buffer,
            primary_transfer_command_buffer,
        });

        todo!()
    }

    fn extension_for_window(window: &RawWindowHandle) -> Result<vk::Extension, HexilError> {
        match window {
            RawWindowHandle::UiKit(_) => Ok(vk::EXT_METAL_SURFACE_EXTENSION),
            RawWindowHandle::AppKit(_) => Ok(vk::EXT_METAL_SURFACE_EXTENSION),
            RawWindowHandle::OhosNdk(_) => Ok(vk::OHOS_SURFACE_EXTENSION),
            RawWindowHandle::Xlib(_) => Ok(vk::KHR_XLIB_SURFACE_EXTENSION),
            RawWindowHandle::Xcb(_) => Ok(vk::KHR_XCB_SURFACE_EXTENSION),
            RawWindowHandle::Wayland(_) => Ok(vk::KHR_WAYLAND_SURFACE_EXTENSION),
            RawWindowHandle::Win32(_) => Ok(vk::KHR_WIN32_SURFACE_EXTENSION),
            RawWindowHandle::AndroidNdk(_) => Ok(vk::KHR_ANDROID_SURFACE_EXTENSION),
            _ => Err(HexilError::UnsupportedPlatform(Backtrace::capture())),
        }
    }

    fn create_device(
        instance: &Instance,
        user_preferred_device: Option<UniqueVulkanId>,
    ) -> Result<(Device, QueueWithInfo, QueueWithInfo)> {
        let physical_device = Self::select_physical_device(instance, user_preferred_device)?;

        let q_fam_props =
            unsafe { instance.get_physical_device_queue_family_properties2(physical_device) };

        let graphics_queue = q_fam_props
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                use std::cmp::Ordering::*;
                match (
                    a.queue_family_properties
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS),
                    a.queue_family_properties
                        .queue_flags
                        .contains(vk::QueueFlags::TRANSFER),
                    b.queue_family_properties
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS),
                    b.queue_family_properties
                        .queue_flags
                        .contains(vk::QueueFlags::TRANSFER),
                ) {
                    (true, _, false, _) => Greater,
                    (false, _, true, _) => Less,
                    (true, true, true, false) => Less,
                    (true, false, true, true) => Greater,
                    _ => a
                        .queue_family_properties
                        .queue_count
                        .cmp(&b.queue_family_properties.queue_count),
                }
            })
            .map(|(idx, _)| idx)
            .expect("At least one queue surely exists");
        let transfer_queue = q_fam_props
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                use std::cmp::Ordering::*;
                match (
                    a.queue_family_properties
                        .queue_flags
                        .contains(vk::QueueFlags::TRANSFER),
                    a.queue_family_properties
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS),
                    b.queue_family_properties
                        .queue_flags
                        .contains(vk::QueueFlags::TRANSFER),
                    b.queue_family_properties
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS),
                ) {
                    (true, _, false, _) => Greater,
                    (false, _, true, _) => Less,
                    (true, true, true, false) => Less,
                    (true, false, true, true) => Greater,
                    _ => a
                        .queue_family_properties
                        .queue_count
                        .cmp(&b.queue_family_properties.queue_count),
                }
            })
            .map(|(idx, _)| idx)
            .expect("At least one queue surely exists");

        let gfx_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue.try_into().unwrap())
            .queue_priorities(if graphics_queue == transfer_queue {
                &[1.0, 0.9]
            } else {
                &[1.0]
            });
        let xfer_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(transfer_queue.try_into().unwrap())
            .queue_priorities(&[0.9]);

        let required_device_extensions = REQUIRED_EXTENSIONS
            .iter()
            .filter_map(|ext| match ext.type_ {
                "device" => Some(ext.name.as_ptr()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let two_qs = [gfx_queue_create_info, xfer_queue_create_info];

        let mut vulkan_1_1_features = vk::PhysicalDeviceVulkan11Features::builder()
            .shader_draw_parameters(true)
            .build();

        let mut vulkan_1_2_features = vk::PhysicalDeviceVulkan12Features::builder()
            .buffer_device_address(true)
            .build();

        let mut vulkan_1_3_features = vk::PhysicalDeviceVulkan13Features::builder()
            .dynamic_rendering(true)
            .inline_uniform_block(true)
            .maintenance4(true)
            .synchronization2(true)
            .shader_demote_to_helper_invocation(true)
            .shader_terminate_invocation(true)
            .build();
        let mut enabled_features = vk::PhysicalDeviceFeatures2::builder()
            .push_next(&mut vulkan_1_3_features)
            .features(vk::PhysicalDeviceFeatures::builder());
        let device_create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&required_device_extensions)
            .queue_create_infos(if graphics_queue == transfer_queue {
                &two_qs.as_slice()[1..=1]
            } else {
                &two_qs.as_slice()
            })
            .push_next(&mut enabled_features);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }?;
        let gfx_q_info = vk::DeviceQueueInfo2::builder()
            .queue_family_index(graphics_queue.try_into().unwrap())
            .queue_index(0);
        let gfx_q = unsafe { device.get_device_queue2(&gfx_q_info) };
        let xfer_q_info = vk::DeviceQueueInfo2::builder()
            .queue_family_index(transfer_queue.try_into().unwrap())
            .queue_index(if graphics_queue == transfer_queue {
                1
            } else {
                0
            });
        let xfer_q = unsafe { device.get_device_queue2(&xfer_q_info) };
        Ok((
            device,
            QueueWithInfo {
                queue: gfx_q,
                queue_family_index: gfx_q_info.queue_family_index,
                queue_index: gfx_q_info.queue_index,
            },
            QueueWithInfo {
                queue: xfer_q,
                queue_family_index: xfer_q_info.queue_family_index,
                queue_index: xfer_q_info.queue_index,
            },
        ))
    }

    fn select_physical_device(
        instance: &Instance,
        user_preferred_device: Option<UniqueVulkanId>,
    ) -> Result<vk::PhysicalDevice> {
        let physical_devices = unsafe { instance.enumerate_physical_devices() }?;

        let physical_devices_extension_properties = physical_devices
            .iter()
            .map(|p| unsafe { instance.enumerate_device_extension_properties(*p, None) })
            .collect::<Result<Vec<_>, _>>()?;

        let physical_devices_compatibility = physical_devices_extension_properties.iter().map(
            |physical_device_supported_extensions| {
                REQUIRED_EXTENSIONS
                    .iter()
                    .filter(|required_extension| required_extension.type_ == "device")
                    .map(|required_device_extension| required_device_extension.name)
                    .all(|req_dev_ext_name| {
                        physical_device_supported_extensions
                            .iter()
                            .map(|physical_device_supported_extension_properties| {
                                physical_device_supported_extension_properties.extension_name
                            })
                            .collect::<Vec<_>>()
                            .contains(&req_dev_ext_name)
                    })
            },
        );

        let physical_devices: Vec<_> = physical_devices
            .into_iter()
            .zip(physical_devices_compatibility)
            .filter_map(|(device, is_compatible)| if is_compatible { Some(device) } else { None })
            .collect();

        let mut physical_devices_properties =
            Vec::<vk::PhysicalDeviceProperties2>::with_capacity(physical_devices.len());
        let mut physical_devices_ids =
            Vec::<vk::PhysicalDeviceIDProperties>::with_capacity(physical_devices.len());

        physical_devices_properties
            .spare_capacity_mut()
            .fill_with(|| MaybeUninit::new(vk::PhysicalDeviceProperties2::builder().build()));
        physical_devices_ids
            .spare_capacity_mut()
            .fill_with(|| MaybeUninit::new(vk::PhysicalDeviceIDProperties::builder().build()));
        unsafe { physical_devices_properties.set_len(physical_devices.len()) };
        unsafe { physical_devices_ids.set_len(physical_devices.len()) };

        physical_devices_properties
            .iter_mut()
            .zip(physical_devices_ids.iter_mut())
            .for_each(|(props, id)| {
                *props = vk::PhysicalDeviceProperties2::builder()
                    .push_next(id)
                    .build();
            });

        physical_devices_properties
            .iter_mut()
            .zip(physical_devices.iter())
            .for_each(|(props, dev)| unsafe {
                instance.get_physical_device_properties2(*dev, props)
            });

        let selected_device: usize = match user_preferred_device {
            None => None,
            Some(UniqueVulkanId::LUID(id)) => physical_devices_ids.iter().position(|phys_dev_id| {
                phys_dev_id.device_luid_valid == vk::TRUE && phys_dev_id.device_luid == id
            }),
            Some(UniqueVulkanId::UUID(id)) => physical_devices_ids
                .iter()
                .position(|phys_dev_id| phys_dev_id.device_uuid == id),
        }
        .unwrap_or_else(|| {
            Self::default_physical_device_selection(physical_devices_properties.iter())
        });

        Ok(physical_devices[selected_device])
    }

    fn default_physical_device_selection<'a>(
        physical_devices_properties: impl Iterator<Item = &'a vk::PhysicalDeviceProperties2>,
    ) -> usize {
        physical_devices_properties
            .enumerate()
            .filter(|(_, p)| p.properties.api_version >= vk::make_version(1, 4, 0))
            .max_by_key(|(_, p)| p.properties.limits.max_compute_work_group_size)
            .map(|(idx, _)| idx)
            .expect("No compatible physical devices!")
    }
}
