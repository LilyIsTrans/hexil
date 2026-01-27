use std::{backtrace::Backtrace, collections::VecDeque, mem::MaybeUninit};

use anyhow::anyhow;
use palette::stimulus::IntoStimulus;
use pkg_version::{pkg_version_major, pkg_version_minor, pkg_version_patch};
use raw_window_handle::RawWindowHandle;

use crate::hexil_prelude::all::*;

pub struct GlobalState {
    pub vulkan_state: VulkanState,
    pub window_state: WindowState,
}

pub struct WindowState {
    pub main_window: winit::window::Window,
}

impl WindowState {
    pub fn new(eloop: &winit::event_loop::ActiveEventLoop) -> Result<Self> {
        Ok(Self {
            main_window: eloop.create_window(
                winit::window::WindowAttributes::default()
                    .with_active(true)
                    .with_title("Hexil")
                    .with_visible(false),
            )?,
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

pub struct VulkanState {
    pub required_extensions: Vec<vk::Extension>,
    pub entry: vulkanalia::Entry,
    pub instance: vulkanalia::Instance,
    pub device: vulkanalia::Device,
    pub graphics_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    active_state: Option<ActiveVulkanState>,
    pub present_id: std::sync::atomic::AtomicU64,
}

impl GlobalState {
    pub fn new(eloop: &winit::event_loop::ActiveEventLoop) -> Result<Self> {
        use raw_window_handle::HasWindowHandle;
        let window_state = WindowState::new(eloop)?;

        let vulkan_state = VulkanState::new(&window_state.main_window.window_handle()?.as_raw())?;

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
            unsafe {
                self.device
                    .destroy_command_pool(active_state.transfer_command_pool, None)
            };
        }
        self.active_state = None;

        unsafe { self.device.destroy_device(None) };

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
            present_id: 0.into(),
        })
    }

    fn get_active_state(&mut self) -> Option<&mut ActiveVulkanState> {
        self.active_state.as_mut()
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
    ) -> Result<(Device, vk::Queue, vk::Queue)> {
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
        let gfx_q = unsafe {
            device.get_device_queue2(
                &vk::DeviceQueueInfo2::builder()
                    .queue_family_index(graphics_queue.try_into().unwrap())
                    .queue_index(0),
            )
        };
        let xfer_q = unsafe {
            device.get_device_queue2(
                &vk::DeviceQueueInfo2::builder()
                    .queue_family_index(transfer_queue.try_into().unwrap())
                    .queue_index(if graphics_queue == transfer_queue {
                        1
                    } else {
                        0
                    }),
            )
        };
        Ok((device, gfx_q, xfer_q))
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

    pub fn next_present_id(&self) -> u64 {
        self.present_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed) // This is wrapping, which is in principle exactly what we want (even if in practice it almost certainly doesn't matter at all)
    }
}
