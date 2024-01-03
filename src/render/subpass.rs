use super::RendererError;
use tracing::instrument;
use try_log::log_tries;
use vulkano as vk;

#[instrument(skip_all)]
#[log_tries(tracing::error)]
pub(super) fn make_canvas_subpasses() -> Result<vk::render_pass::SubpassDescription, RendererError>
{
    use vk::image as img;
    use vk::render_pass as rpass;

    // Palette
    let palette_attachment = rpass::AttachmentReference {
        attachment: 0,
        layout: img::ImageLayout::ShaderReadOnlyOptimal,
        aspects: img::ImageAspect::Color.into(),
        ..Default::default()
    };

    // // Canvas base colours as indices into the palette
    // let canvas_base_attachment = rpass::AttachmentReference {
    //     attachment: 1,
    //     layout: img::ImageLayout::ShaderReadOnlyOptimal,
    //     aspects: img::ImageAspect::Color.into(),
    //     ..Default::default()
    // };

    let output_attachment = rpass::AttachmentReference {
        attachment: 1,
        layout: img::ImageLayout::ColorAttachmentOptimal,
        aspects: img::ImageAspect::Color.into(),
        ..Default::default()
    };

    let spass = rpass::SubpassDescription {
        flags: rpass::SubpassDescriptionFlags::empty(),
        view_mask: 0,
        input_attachments: vec![Some(palette_attachment)],
        color_attachments: vec![Some(output_attachment)],
        preserve_attachments: vec![],
        ..Default::default()
    };

    Ok(spass)
}