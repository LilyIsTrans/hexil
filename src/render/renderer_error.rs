use std::sync::mpsc::RecvTimeoutError;
use thiserror::Error;
use vk::pipeline::layout::IntoPipelineLayoutCreateInfoError;
use vulkano as vk;

/// The unified error type for all Renderer functions. There are so many different things that can go wrong with them, it's best to just
/// shove them all into one massive error type.
#[derive(Error, Debug)]
pub enum RendererError {
    #[error(transparent)]
    VkLoadErr(#[from] vk::LoadingError),
    #[error(transparent)]
    VkErr(#[from] vk::VulkanError),
    #[error(transparent)]
    ValidVkErr(#[from] vk::Validated<vk::VulkanError>),
    #[error(transparent)]
    ValidBufErr(#[from] vk::Validated<vk::buffer::AllocateBufferError>),
    #[error(transparent)]
    WindowHandleError(#[from] winit::raw_window_handle::HandleError),
    #[error("No physical devices? At all!? Seriously, as far as this program can tell, you must be reading this through a serial port, which like, props, but what on earth made you think a pixel art program would work with that?")]
    NoPhysicalDevices,
    #[error(transparent)]
    ChannelError(#[from] RecvTimeoutError),
    #[error("No graphics queues available!")]
    NoGraphicsQueues,
    #[error("No transfer queues available!")]
    NoTransferQueues,
    #[error(transparent)]
    VkValidationErr(#[from] Box<vk::ValidationError>),
    #[error(transparent)]
    VkCommandBufExecErr(#[from] vk::command_buffer::CommandBufferExecError),
    #[error("At least one subpass must be specified!")]
    NoSubpassesSpecifiedForRenderpass,
    #[error("Shader source file not found!")]
    ShaderSourceNotFound,
    #[error(transparent)]
    PipelineCreateInfoErr(#[from] IntoPipelineLayoutCreateInfoError),
    #[error(transparent)]
    RecvErr(#[from] std::sync::mpsc::RecvError),
}
