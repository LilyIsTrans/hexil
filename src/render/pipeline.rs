use vulkano as vk;

use super::RendererError;

use super::Renderer;

impl Renderer {
    pub(super) fn make_pipeline(
        &mut self,
        tile_vertex_count: usize,
        tile_index_count: usize,
    ) -> Result<vk::pipeline::GraphicsPipeline, RendererError> {
        use pip::graphics as gfx;
        use vk::descriptor_set as ds;
        use vk::pipeline as pip;

        let ds_type = ds::layout::DescriptorType::UniformBuffer;
        let mut ds_vertex_bindings =
            ds::layout::DescriptorSetLayoutBinding::descriptor_type(ds_type);
        ds_vertex_bindings.descriptor_count = tile_vertex_count
            .try_into()
            .expect("I sure hope a usize fits in a u32");
        ds_vertex_bindings.binding_flags = ds::layout::DescriptorBindingFlags::empty();
        let mut ds_vertex_index_bindings = ds_vertex_bindings.clone();
        ds_vertex_index_bindings.descriptor_count = tile_index_count
            .try_into()
            .expect("I sure hope a usize fits into a u32");
        let ds_layout = ds::layout::DescriptorSetLayoutCreateInfo {
            bindings: todo!(),
            ..Default::default()
        };

        todo!()
    }
}
