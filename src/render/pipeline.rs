use vk::descriptor_set::allocator::DescriptorSetAllocator;
use vulkano as vk;

use super::renderer_error::RendererError;

use super::Renderer;

use tracing::instrument;

impl Renderer {
    /// WIP. Wraps the process of creating a Vulkan graphics pipeline.
    #[instrument(skip(self))]
    pub(super) fn make_pipeline(
        &mut self,
        tile_vertex_count: usize,
        tile_index_count: usize,
        hexagon_mode: bool,
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
        ds_vertex_bindings.stages = vk::shader::ShaderStages::VERTEX;

        ds_vertex_bindings.binding_flags = ds::layout::DescriptorBindingFlags::empty();

        let mut ds_vertex_index_bindings = ds_vertex_bindings.clone();
        ds_vertex_index_bindings.descriptor_count = tile_index_count
            .try_into()
            .expect("I sure hope a usize fits into a u32");

        let mut base_tile_bindings = std::collections::BTreeMap::new();
        base_tile_bindings.insert(0, ds_vertex_bindings);
        base_tile_bindings.insert(1, ds_vertex_index_bindings);

        let ds_layout_base_tile = ds::layout::DescriptorSetLayoutCreateInfo {
            bindings: base_tile_bindings,
            ..Default::default()
        };
        let ds_layout_base_tile =
            ds::layout::DescriptorSetLayout::new(self.logical_device.clone(), ds_layout_base_tile)?;

        let mut ds_allocator = ds::allocator::StandardDescriptorSetAllocator::new(
            self.logical_device.clone(),
            Default::default(),
        );
        let buf = if hexagon_mode {
            self.hex_buf.clone()
        } else {
            self.square_buf.clone()
        };

        Ok(todo!())
    }
}
