use crate::hexil_prelude::all::*;
use anyhow::anyhow;
use tracing::{debug, error, info, instrument, warn};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

impl super::GlobalState {
    pub fn handle_resized(
        &mut self,
        PhysicalSize { width, height }: PhysicalSize<u32>,
        _window_id: WindowId,
    ) -> Result<()> {
        self.window_state
            .main_window
            .rebuild_swapchain(&self.vulkan_state, (width.try_into()?, height.try_into()?))?;
        Ok(())
    }

    pub fn handle_close_requested(&mut self, event_loop: &ActiveEventLoop) {
        match self.window_state.main_window.surface.as_mut() {
            None => (),
            Some(surface) => {
                match surface.swapchain.as_mut() {
                    None => (),
                    Some(swapchain) => {
                        unsafe {
                            self.vulkan_state
                                .device
                                .destroy_swapchain_khr(swapchain.swapchain, None)
                        };
                        swapchain.swapchain = vk::Handle::null();
                    }
                };
                surface.swapchain = None;
                unsafe {
                    self.vulkan_state
                        .instance
                        .destroy_surface_khr(surface.surface, None)
                };
                surface.surface = vk::Handle::null();
            }
        }
        self.window_state.main_window.surface = None;
        event_loop.exit();
    }

    pub fn handle_destroyed(&mut self, event_loop: &ActiveEventLoop) {
        self.handle_close_requested(event_loop)
    }

    pub fn perform_redraw(&mut self) -> Result<()> {
        let surface = self
            .window_state
            .main_window
            .get_or_create_surface(&self.vulkan_state)?;
        let mut swapchain = match surface.swapchain.as_mut() {
            Some(swapchain) => swapchain,
            None => surface.rebuild_swapchain(
                &self.vulkan_state,
                (
                    surface.size.width.try_into()?,
                    surface.size.height.try_into()?,
                ),
            )?,
        };

        (unsafe {
            self.vulkan_state
                .device
                .reset_fences(&[swapchain.acquire_fence])
        })?;

        // Safety: semaphore and fence must not both be null, and each of them must either be null or unsignalled and not in use by any other operation
        let image_idx = loop {
            match unsafe {
                self.vulkan_state.device.acquire_next_image_khr(
                    swapchain.swapchain,
                    u64::MAX,
                    vk::Semaphore::null(),
                    swapchain.acquire_fence,
                )
            } {
                Ok((idx, vk::SuccessCode::SUCCESS)) => break idx,
                Ok((_, vk::SuccessCode::TIMEOUT)) => continue,
                Ok((_, vk::SuccessCode::NOT_READY)) => {
                    warn!("Improper timeout support for acquiring swapchain images!");
                    continue;
                }
                Ok((_, vk::SuccessCode::SUBOPTIMAL_KHR)) | Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                    swapchain = match surface.swapchain.as_mut() {
                        Some(swapchain) => swapchain,
                        None => surface.rebuild_swapchain(
                            &self.vulkan_state,
                            (
                                surface.size.width.try_into()?,
                                surface.size.height.try_into()?,
                            ),
                        )?,
                    };
                    warn!(
                        "Getting out of date swapchain images! Seeing this a few times, especially when resizing or moving windows, is normal, but if you see LOTS of these and are having serious performance problems or unpredictable crashes, this is probably why."
                    );
                    continue;
                }
                Ok((_, code)) => {
                    error!(
                        "Unexpected success code: {:?} from vkAcquireNextImageKHR! Something is SERIOUSLY borked!! Crashing ASAP to avoid corrupting your graphics driver; sorry if you lose any work, there was unfortunately no other way.",
                        code
                    );
                    panic!("We're thoroughly borked! Shutdown NOW!")
                }
                Err(
                    e @ vk::ErrorCode::OUT_OF_DEVICE_MEMORY | e @ vk::ErrorCode::OUT_OF_HOST_MEMORY,
                ) => {
                    error!(
                        "Unable to redraw due to memory pressure ({:?})! Attempting graceful shutdown.",
                        e
                    );
                    return Err(anyhow!(e));
                }
                Err(vk::ErrorCode::FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT) => {
                    unreachable!("We don't use exclusive fullscreen.")
                }
                Err(e @ vk::ErrorCode::SURFACE_LOST_KHR) => {
                    error!(
                        "The surface was lost before redraw could finish! Attempting graceful shutdown."
                    );
                    return Err(anyhow!(e));
                }
                Err(e @ vk::ErrorCode::DEVICE_LOST) => {
                    error!(
                        "Lost handle to Vulkan device!! Something has gone *terribly* wrong! Attempting graceful shutdown."
                    );
                    return Err(anyhow!(e));
                }
                Err(e @ vk::ErrorCode::VALIDATION_FAILED) => {
                    error!(
                        "A validation error has occurred! This is Hexil's fault, please report a bug to `hexil@lilymccabe.ca`. Attempting graceful shutdown."
                    );
                    return Err(anyhow!(e));
                }
                Err(e @ vk::ErrorCode::UNKNOWN) => {
                    error!(
                        "An unknown vulkan error has occurred! This is probably Hexil's fault, possibly your graphics driver's fault. Please report a bug to `hexil@lilymccabe.ca`. Attempting graceful shutdown."
                    );
                    return Err(anyhow!(e));
                }
                Err(e) => {
                    error!(
                        "An unexpected type of Vulkan error has occurred while acquiring swapchain images! ({:?}). Something has gone terribly wrong! Attempting graceful shutdown.",
                        e
                    );
                    return Err(anyhow!(e));
                }
            }
        };

        let image = swapchain.images[TryInto::<usize>::try_into(image_idx)?];

        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .format(surface.swapchain_format.clone())
            .image(image)
            .components(vk::ComponentMapping::default())
            .view_type(vk::ImageViewType::_2D)
            .subresource_range(vk::ImageSubresourceRange::default());

        let image_view = unsafe {
            self.vulkan_state
                .device
                .create_image_view(&image_view_create_info, None)
        }?;

        let color_attachements = [vk::RenderingAttachmentInfo::builder()
            .image_view(image_view)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .clear_value(vk::ClearValue {
                color: (vk::ClearColorValue {
                    float32: [1.0f32; 4],
                }),
            })
            .image_layout(vk::ImageLayout::UNDEFINED)
            .resolve_mode(vk::ResolveModeFlags::NONE)];

        let render_info = vk::RenderingInfo::builder()
            .color_attachments(&color_attachements)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: surface.size,
            });

        let active_vulkan_state = *self.vulkan_state.get_or_activate()?;

        let tmp = [swapchain.acquire_fence];

        unsafe {
            self.vulkan_state.device.wait_for_fences(
                &tmp,
                true,
                std::time::Duration::from_millis(5)
                    .as_nanos()
                    .try_into()
                    .unwrap(),
            )
        };

        unsafe {
            self.vulkan_state.device.cmd_begin_rendering(
                active_vulkan_state.primary_graphics_command_buffer,
                &render_info,
            )
        };

        todo!()
    }
}
