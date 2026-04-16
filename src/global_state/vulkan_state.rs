use std::backtrace::Backtrace;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

use itertools::Itertools;
use parking_lot::Mutex;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use pkg_version::pkg_version_major;
use pkg_version::pkg_version_minor;
use pkg_version::pkg_version_patch;
use raw_window_handle::RawWindowHandle;
use vulkanalia::vk;

use super::QueueWithInfo;
use crate::hexil_prelude::all::*;
use anyhow::Result;
use anyhow::anyhow;

unsafe impl Send for ActiveVulkanState {}
impl !Sync for ActiveVulkanState {}

impl ActiveVulkanState {
    /// The returned fence must be signalled to indicate that the caller has returned the command buffer to the manager. As long as it remains unsignalled, the caller may reuse the returned command buffer as much as desired.
    pub fn get_graphics_command_buffer(
        &self,
        device: &vulkanalia::Device,
    ) -> Result<(vk::CommandBuffer, vk::Fence)> {
        let buffer_pool = self.primary_graphics_command_buffers.lock();

        (unsafe { device.wait_for_fences(buffer_pool.fence_slice(), false, u64::MAX) })?;

        let index = buffer_pool
            .fence_slice()
            .into_iter()
            .map(|fence| unsafe { device.get_fence_status(*fence) })
            .process_results(|mut codes| codes.position(|code| code == vk::SuccessCode::SUCCESS))?
            .expect("Due to the previous wait_for_fences operation, along with the Mutex lock, it should be impossible for no fence to be signalled.");

        unsafe { device.reset_fences(&buffer_pool.fence_slice()[index..=index]) }?;
        (unsafe {
            device.reset_command_buffer(
                buffer_pool.command_buffer_slice()[index],
                vk::CommandBufferResetFlags::empty(),
            )
        })?;

        Ok((
            buffer_pool.command_buffer_slice()[index],
            buffer_pool.fence_slice()[index],
        ))
    }
}

#[derive(Debug)]
struct CommandBufferArrayWithSyncStatus {
    size: usize,
    buffers: NonNull<vk::CommandBuffer>,
    ready_fences: NonNull<vk::Fence>,
}

impl CommandBufferArrayWithSyncStatus {
    pub fn new(
        command_pool: vk::CommandPool,
        device: &vulkanalia::Device,
        size: u32,
    ) -> Result<Self> {
        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(size);

        let pending_fences: *mut vk::Fence = unsafe {
            std::alloc::alloc(
                std::alloc::Layout::array::<vk::Fence>(size.try_into()?)?
                    .align_to(std::mem::align_of::<vk::Fence>())?,
            )
            .cast()
        };

        let ready_fences =
            NonNull::new(pending_fences).ok_or_else(|| anyhow!("Allocation failure!"))?;

        let mut buffers = unsafe { device.allocate_command_buffers(&info) }?;
        buffers.shrink_to_fit();
        let (buffers, _, _) = buffers.into_raw_parts();
        // SAFETY: buffers is coming straight from an initialized Vec, so it's definitely aligned and NonNull
        let buffers = unsafe { NonNull::new_unchecked(buffers) };

        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for fence in (0..size).map(|idx| unsafe { ready_fences.offset(idx.try_into().unwrap()) }) {
            unsafe {
                fence.write(device.create_fence(&fence_info, None)?);
            };
        }

        Ok(Self {
            size: size.try_into()?,
            buffers,
            ready_fences,
        })
    }

    pub const fn command_buffer_slice(&self) -> &[vk::CommandBuffer] {
        unsafe { std::slice::from_raw_parts(self.buffers.as_ptr().cast_const(), self.size) }
    }

    pub const fn fence_slice(&self) -> &[vk::Fence] {
        unsafe { std::slice::from_raw_parts(self.ready_fences.as_ptr().cast_const(), self.size) }
    }
}

#[derive(Debug)]
pub struct ActiveVulkanState {
    graphics_command_pool: vk::CommandPool,
    transfer_command_pool: vk::CommandPool,

    primary_graphics_command_buffers: Mutex<CommandBufferArrayWithSyncStatus>,
}

#[derive(Debug)]
pub struct VulkanState {
    required_extensions: Vec<vk::Extension>,
    entry: vulkanalia::Entry,
    instance: vulkanalia::Instance,
    device: vulkanalia::Device,
    graphics_queue: RwLock<QueueWithInfo>,
    transfer_queue: RwLock<QueueWithInfo>,
    active_state: ActiveVulkanState,
}

impl VulkanState {
    pub fn required_extensions(&self) -> &[vk::Extension] {
        &self.required_extensions
    }

    pub fn entry(&self) -> &vulkanalia::Entry {
        &self.entry
    }

    pub fn instance(&self) -> &vulkanalia::Instance {
        &self.instance
    }

    pub fn device(&self) -> &vulkanalia::Device {
        &self.device
    }

    pub fn graphics_queue(&self) -> &RwLock<QueueWithInfo> {
        &self.graphics_queue
    }

    pub fn transfer_queue(&self) -> &RwLock<QueueWithInfo> {
        &self.transfer_queue
    }

    pub fn active_state(&self) -> &ActiveVulkanState {
        &self.active_state
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
                b"VK_LAYER_KHRONOS_validation\0".as_slice(),
            )
            .as_ptr()];
        #[cfg(debug_assertions)]
        let info = info.enabled_layer_names(&VALIDATION_LAYERS_NAME);

        let instance = unsafe { entry.create_instance(&info, None) }?;

        let device = Self::create_device(&instance, None)?;
        let graphics_queue = RwLock::new(device.1);
        let transfer_queue = RwLock::new(device.2);
        let device = device.0;
        let active_state = Self::activate(&device, graphics_queue.read(), transfer_queue.read())?;
        Ok(Self {
            required_extensions,
            entry,
            instance,
            device,
            graphics_queue,
            transfer_queue,
            active_state,
        })
    }

    fn activate(
        device: &vulkanalia::Device,
        graphics_queue: RwLockReadGuard<QueueWithInfo>,
        transfer_queue: RwLockReadGuard<QueueWithInfo>,
    ) -> Result<ActiveVulkanState> {
        let gfx_cmd_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_queue.queue_family_index);
        let xfer_cmd_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(transfer_queue.queue_family_index);

        let graphics_command_pool =
            unsafe { device.create_command_pool(&gfx_cmd_pool_create_info, None) }?;
        let transfer_command_pool =
            unsafe { device.create_command_pool(&xfer_cmd_pool_create_info, None) }?;

        let active_state =
            ActiveVulkanState {
                graphics_command_pool,
                transfer_command_pool,
                primary_graphics_command_buffers: Mutex::new(
                    CommandBufferArrayWithSyncStatus::new(graphics_command_pool, &device, 8)?,
                ),
            };
        Ok(active_state)
    }
}

impl VulkanState {
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
            .push_next(&mut vulkan_1_2_features)
            .push_next(&mut vulkan_1_1_features)
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
impl Drop for VulkanState {
    fn drop(&mut self) {
        let device: &vulkanalia::Device = &self.device;
        unsafe {
            device.destroy_command_pool((&mut self.active_state).graphics_command_pool, None)
        };
        self.active_state.graphics_command_pool = vk::Handle::null();
        unsafe {
            device.destroy_command_pool((&mut self.active_state).transfer_command_pool, None)
        };
        self.active_state.transfer_command_pool = vk::Handle::null();

        unsafe { self.device.destroy_device(None) };
        self.graphics_queue.write().queue = vk::Handle::null();
        self.transfer_queue.write().queue = vk::Handle::null();

        unsafe { self.instance.destroy_instance(None) };
    }
}
