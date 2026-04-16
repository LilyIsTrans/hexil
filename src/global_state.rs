use anyhow::anyhow;
use std::{num::NonZeroU32, sync::atomic::AtomicU64};
use tracing::{debug, error, instrument};
use vulkanalia::vk::{SurfaceFormat2KHR, SurfaceFormatKHR};

use crate::{global_state::vulkan_state::VulkanState, hexil_prelude::all::*};

pub struct GlobalState {
    pub vulkan_state: VulkanState,
    pub window_state: WindowState,
}

pub(crate) mod handle_events;

#[derive(Debug)]
#[repr(transparent)]
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

#[derive(Debug)]
pub struct Surface {
    pub surface: vk::SurfaceKHR,
    pub swapchain: Swapchain,
    pub swapchain_format: vk::Format,
    pub selected_color_space: vk::ColorSpaceKHR,
    pub selected_alpha_composite_mode: vk::CompositeAlphaFlagsKHR,
    pub selected_present_mode: vk::PresentModeKHR,
    pub min_image_count: u32,
    pub size: vk::Extent2D,
}

impl Surface {
    pub fn rebuild_swapchain(
        &mut self,
        vulkan_state: &vulkan_state::VulkanState,
        new_size: (NonZeroU32, NonZeroU32),
    ) -> Result<()> {
        self.swapchain = Swapchain::build_swapchain(
            self.surface,
            self.min_image_count,
            self.swapchain_format,
            self.selected_color_space,
            self.selected_alpha_composite_mode,
            self.selected_present_mode,
            Some(&self.swapchain),
            vulkan_state,
            new_size,
        )?;
        Ok(())
    }
}

impl Swapchain {
    const SHARING_MODE: vk::SharingMode = vk::SharingMode::EXCLUSIVE;
    /// Call this whenever the size of the window has changed (including from, but not to, nonexistence).
    pub fn build_swapchain(
        surface: vk::SurfaceKHR,
        min_image_count: u32,
        swapchain_format: vk::Format,
        selected_color_space: vk::ColorSpaceKHR,
        selected_alpha_composite_mode: vk::CompositeAlphaFlagsKHR,
        selected_present_mode: vk::PresentModeKHR,
        old_swapchain: Option<&Swapchain>,
        vulkan_state: &vulkan_state::VulkanState,
        new_size: (NonZeroU32, NonZeroU32),
    ) -> Result<Swapchain> {
        let info = vk::SwapchainCreateInfoKHR::builder()
            .flags(
                vk::SwapchainCreateFlagsKHR::DEFERRED_MEMORY_ALLOCATION
                    | vk::SwapchainCreateFlagsKHR::PRESENT_WAIT_2
                    | vk::SwapchainCreateFlagsKHR::PRESENT_ID_2,
            )
            .surface(surface)
            .min_image_count(min_image_count)
            .image_format(swapchain_format)
            .image_color_space(selected_color_space)
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
            .image_color_space(selected_color_space)
            .composite_alpha(selected_alpha_composite_mode)
            .present_mode(selected_present_mode)
            .clipped(true)
            .old_swapchain(
                old_swapchain
                    .as_ref()
                    .map_or(vk::SwapchainKHR::null(), |s| s.swapchain),
            );

        // Safety: All fields in `info` are set to valid values
        let swapchain = unsafe { vulkan_state.device().create_swapchain_khr(&info, None) }?;

        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::empty());

        let fence = unsafe { vulkan_state.device().create_fence(&fence_info, None) }?;

        let images = unsafe { vulkan_state.device().get_swapchain_images_khr(swapchain) }?.into();

        Ok(Swapchain {
            swapchain,
            images,
            acquire_fence: fence,
            last_present_id: PresentID::ZERO,
        })
    }
}

#[derive(Debug)]
pub struct Swapchain {
    pub swapchain: vk::SwapchainKHR,
    pub images: Box<[vk::Image]>,
    pub acquire_fence: vk::Fence,
    pub last_present_id: PresentID,
}

#[derive(Debug)]
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
#[instrument(level = "info")]
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

#[instrument(level = "info")]
fn select_composite_alpha_mode(modes: vk::CompositeAlphaFlagsKHR) -> vk::CompositeAlphaFlagsKHR {
    if modes.contains(vk::CompositeAlphaFlagsKHR::OPAQUE) {
        vk::CompositeAlphaFlagsKHR::OPAQUE
    } else if modes.contains(vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED) {
        vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED
    } else if modes.contains(vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED) {
        vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED
    } else if modes.contains(vk::CompositeAlphaFlagsKHR::INHERIT) {
        vk::CompositeAlphaFlagsKHR::INHERIT
    } else {
        panic!("No known composite alpha mode supported!")
    }
}

impl HexilWindow {
    #[instrument(level = "info")]
    pub fn create_surface(&mut self, vulkan_state: &VulkanState) -> Result<&mut Surface> {
        use vulkanalia::window::create_surface;

        let surface =
            (unsafe { create_surface(&vulkan_state.instance(), &self.window, &self.window) })?;

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
                .instance()
                .get_physical_device_surface_capabilities2_khr(
                    vulkan_state.device().physical_device(),
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
                .instance()
                .get_physical_device_surface_capabilities2_khr(
                    vulkan_state.device().physical_device(),
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
                .instance()
                .get_physical_device_surface_formats2_khr(
                    vulkan_state.device().physical_device(),
                    &surface_info,
                )
        }?;

        let SurfaceFormat2KHR {
            s_type: _,
            next: _,
            surface_format:
                SurfaceFormatKHR {
                    format: swapchain_format,
                    color_space: selected_color_space,
                },
        } = surface_formats
            .iter()
            .copied()
            .max_by(compare_surface_formats)
            .expect("No surface formats supported!");

        let selected_alpha_composite_mode = select_composite_alpha_mode(
            surface_capabilities
                .surface_capabilities
                .supported_composite_alpha,
        );

        let min_image_count = surface_capabilities.surface_capabilities.min_image_count;

        let size = surface_capabilities.surface_capabilities.current_extent;
        let surface = Surface {
            surface,
            swapchain: Swapchain::build_swapchain(
                surface,
                min_image_count,
                swapchain_format,
                selected_color_space,
                selected_alpha_composite_mode,
                selected_present_mode,
                None,
                vulkan_state,
                (size.width.try_into()?, size.height.try_into()?),
            )?,
            swapchain_format,
            selected_color_space,
            selected_alpha_composite_mode,
            selected_present_mode,
            min_image_count,
            size,
        };

        self.surface = Some(surface);

        Ok(self
            .surface
            .as_mut()
            .expect("We literally just created the surface."))
    }

    fn rebuild_swapchain(
        &mut self,
        vulkan_state: &VulkanState,
        _new_size: (NonZeroU32, NonZeroU32),
    ) -> Result<&mut Swapchain> {
        Ok(&mut self.get_or_create_surface(vulkan_state)?.swapchain)
    }

    fn get_or_create_surface(&mut self, vulkan_state: &VulkanState) -> Result<&mut Surface> {
        match self.surface {
            Some(_) => Ok(self.surface.as_mut().unwrap()),
            None => self.create_surface(vulkan_state),
        }
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

#[derive(Clone, Copy, Debug)]
pub struct QueueWithInfo {
    pub queue: vk::Queue,
    pub queue_family_index: u32,
    pub queue_index: u32,
}

pub mod vulkan_state;

impl GlobalState {
    /// Initializes Hexil's global state
    ///
    /// This will initialize both the window system state [`window_state`] and the
    /// vulkan library/graphics card state. This includes creating the main window,
    /// though nothing will be drawn to it.
    pub fn new(eloop: &winit::event_loop::ActiveEventLoop) -> Result<Self> {
        use raw_window_handle::HasWindowHandle;
        let window_state = WindowState::new(eloop)?;

        let vulkan_state = vulkan_state::VulkanState::new(
            &window_state.main_window.window.window_handle()?.as_raw(),
        )?;

        Ok(Self {
            vulkan_state,
            window_state,
        })
    }
}
