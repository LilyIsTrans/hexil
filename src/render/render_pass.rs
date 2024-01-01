use std::sync::Arc;
use vulkano as vk;

type Result<T> = std::result::Result<T, super::RendererError>;

pub(super) fn make_renderpass(
    device: Arc<vk::device::Device>,
    subpass_descriptions: Vec<vk::render_pass::SubpassDescription>,
    attachments: Vec<vk::render_pass::AttachmentDescription>,
) -> Result<Arc<vk::render_pass::RenderPass>> {
    use vk::render_pass as rpass;

    #[cfg(debug_assertions)]
    {
        if subpass_descriptions.is_empty() {
            return Err(super::RendererError::NoSubpassesSpecifiedForRenderpass);
        };
    };

    let info = rpass::RenderPassCreateInfo {
        flags: rpass::RenderPassCreateFlags::empty(),
        attachments,
        subpasses: subpass_descriptions,
        dependencies: Vec::new(),
        correlated_view_masks: Vec::new(),
        ..Default::default()
    };

    let pass = rpass::RenderPass::new(device, info)?;

    Ok(pass)
}
