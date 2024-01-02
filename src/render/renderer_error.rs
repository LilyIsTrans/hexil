use vk::pipeline::layout::IntoPipelineLayoutCreateInfoError;
use vulkano as vk;

use std::sync::mpsc::RecvTimeoutError;
use thiserror::Error;

/// The unified error type for all Renderer functions. There are so many different things that can go wrong with them, it's best to just
/// shove them all into one massive error type.
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
    #[error("At least one subpass must be specified!")]
    NoSubpassesSpecifiedForRenderpass,
    #[error("Shader source file not found!")]
    ShaderSourceNotFound,
    #[error("{0}")]
    PipelineCreateInfoErr(IntoPipelineLayoutCreateInfoError),
    #[error("{0}")]
    RecvErr(std::sync::mpsc::RecvError),
}

impl From<std::sync::mpsc::RecvError> for RendererError {
    #[instrument(level = "error")]
    fn from(v: std::sync::mpsc::RecvError) -> Self {
        Self::RecvErr(v)
    }
}

use tracing::instrument;

impl From<IntoPipelineLayoutCreateInfoError> for RendererError {
    #[instrument(level = "error")]
    fn from(v: IntoPipelineLayoutCreateInfoError) -> Self {
        Self::PipelineCreateInfoErr(v)
    }
}

impl From<RecvTimeoutError> for RendererError {
    #[instrument(level = "error")]
    fn from(v: RecvTimeoutError) -> Self {
        Self::ChannelError(v)
    }
}

impl From<vk::command_buffer::CommandBufferExecError> for RendererError {
    #[instrument(level = "error")]
    fn from(v: vk::command_buffer::CommandBufferExecError) -> Self {
        Self::VkCommandBufExecErr(v)
    }
}

impl From<Box<vk::ValidationError>> for RendererError {
    #[instrument(level = "error")]
    fn from(v: Box<vk::ValidationError>) -> Self {
        Self::VkValidationErr(v)
    }
}

impl From<vk::Validated<vk::buffer::AllocateBufferError>> for RendererError {
    #[instrument(level = "error")]
    fn from(v: vk::Validated<vk::buffer::AllocateBufferError>) -> Self {
        Self::ValidBufErr(v)
    }
}

impl From<winit::raw_window_handle::HandleError> for RendererError {
    #[instrument(level = "error")]
    fn from(v: winit::raw_window_handle::HandleError) -> Self {
        Self::WindowHandleError(v)
    }
}

impl From<vk::Validated<vk::VulkanError>> for RendererError {
    #[instrument(level = "error")]
    fn from(v: vk::Validated<vk::VulkanError>) -> Self {
        Self::ValidVkErr(v)
    }
}

impl From<vk::VulkanError> for RendererError {
    #[instrument(level = "error")]
    fn from(v: vk::VulkanError) -> Self {
        Self::VkErr(v)
    }
}

impl From<vk::LoadingError> for RendererError {
    #[instrument(level = "error")]
    fn from(v: vk::LoadingError) -> Self {
        Self::VkLoadErr(v)
    }
}
