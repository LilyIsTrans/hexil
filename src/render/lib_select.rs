use std::sync::Arc;

use vulkano as vk;

use vk::VulkanLibrary;

use super::renderer_error;

use tracing::instrument;

use super::Renderer;

impl Renderer {
    #[instrument]
    pub(crate) fn get_vulkan_library() -> Result<Arc<VulkanLibrary>, renderer_error::RendererError>
    {
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
        Ok(lib)
    }
}
