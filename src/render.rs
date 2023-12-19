use thiserror::Error;
use tracing::instrument;
use vk::VulkanLibrary;
use vulkano as vk;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("{0}")]
    VkLoadErr(vk::LoadingError),
}

impl From<vk::LoadingError> for RendererError {
    fn from(v: vk::LoadingError) -> Self {
        Self::VkLoadErr(v)
    }
}

pub struct Renderer {}

impl Renderer {
    #[instrument]
    pub async fn initialize() -> Result<Self, RendererError> {
        let lib = VulkanLibrary::new()?;

        tracing::info!(
            "Successfully loaded Vulkan version {}.{}.{}",
            lib.api_version().major,
            lib.api_version().minor,
            lib.api_version().patch
        );

        todo!()
    }
}
