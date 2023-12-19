use std::sync::Arc;
use tracing::instrument;

use thiserror::Error;
use tracing::info;
use vk::VulkanLibrary;
use vulkano as vk;
use winit::window::Window;

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
}

pub enum RenderCommand {}

pub async fn render_thread(
    window: Arc<Window>,
    mut channel: tokio::sync::mpsc::Receiver<RenderCommand>,
) -> Result<(), RendererError> {
    let mut renderer = Renderer::initialize(window)?;

    loop {
        match channel.recv().await {
            None => return Ok(()),
            Some(_) => todo!(),
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

        let needed_extensions = vk::instance::InstanceExtensions {
            khr_surface: true,
            // khr_wayland_surface: cfg!(wayland),
            // khr_win32_surface: cfg!(win32),
            // khr_xcb_surface: cfg!(x11),
            // ext_metal_surface: cfg!(metal),
            ext_surface_maintenance1: false, // Might use later
            ext_swapchain_colorspace: false, // Might use later
            ..Default::default()
        };

        let mut info = vk::instance::InstanceCreateInfo::application_from_cargo_toml();
        info.enabled_extensions = needed_extensions;

        info!("Attempting to create instance...");

        let instance = vk::instance::Instance::new(lib, info)?;
        info!("Instance created successfully!");

        let surface = vk::swapchain::Surface::from_window(instance.clone(), window.clone())?;

        Ok(Self {
            instance,
            window,
            surface,
        })
    }
}
