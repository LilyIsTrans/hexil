use std::sync::Arc;

use vulkano as vk;

use super::renderer_error::RendererError;

use super::Renderer;

use tracing::instrument;

impl Renderer {
    /// WIP. Wraps the process of creating a Vulkan graphics pipeline.
    #[instrument]
    pub(super) fn make_gfx_pipeline(
        device: Arc<vk::device::Device>,
        stages: Vec<vk::pipeline::PipelineShaderStageCreateInfo>,
        vertex_input_state: vk::pipeline::graphics::vertex_input::VertexInputState,
        input_assembly_state: vk::pipeline::graphics::input_assembly::InputAssemblyState,
        tessellation_state: Option<vk::pipeline::graphics::tessellation::TessellationState>,
        viewport_state: Option<vk::pipeline::graphics::viewport::ViewportState>,
        rasterization_state: vk::pipeline::graphics::rasterization::RasterizationState,
        multisample_state: Option<vk::pipeline::graphics::multisample::MultisampleState>,
        depth_stencil_state: Option<vk::pipeline::graphics::depth_stencil::DepthStencilState>,
        color_blend_state: Option<vk::pipeline::graphics::color_blend::ColorBlendState>,
        subpass: vk::pipeline::graphics::subpass::PipelineSubpassType,
        discard_rectangle_state: Option<
            vk::pipeline::graphics::discard_rectangle::DiscardRectangleState,
        >,
        set_layouts: Vec<Arc<vk::descriptor_set::layout::DescriptorSetLayout>>,
        push_constant_ranges: Vec<vk::pipeline::layout::PushConstantRange>,
    ) -> Result<Arc<vk::pipeline::GraphicsPipeline>, RendererError> {
        use pip::graphics as gfx;
        use vk::pipeline as pip;

        let layout = pip::layout::PipelineLayoutCreateInfo {
            flags: pip::layout::PipelineLayoutCreateFlags::empty(),
            set_layouts,
            push_constant_ranges,
            ..Default::default()
        };
        let layout = pip::PipelineLayout::new(device.clone(), layout)?;

        let mut pipeline = gfx::GraphicsPipelineCreateInfo::layout(layout);
        pipeline.flags = pip::PipelineCreateFlags::empty();
        pipeline.stages = stages.into();
        pipeline.vertex_input_state = Some(vertex_input_state);
        pipeline.input_assembly_state = Some(input_assembly_state);
        pipeline.tessellation_state = tessellation_state;
        pipeline.viewport_state = viewport_state;
        pipeline.rasterization_state = Some(rasterization_state);
        pipeline.multisample_state = multisample_state;
        pipeline.depth_stencil_state = depth_stencil_state;
        pipeline.color_blend_state = color_blend_state;
        pipeline.subpass = Some(subpass);
        pipeline.discard_rectangle_state = discard_rectangle_state;
        let pipeline = gfx::GraphicsPipeline::new(device, None, pipeline)?;

        Ok(pipeline)
    }
}
