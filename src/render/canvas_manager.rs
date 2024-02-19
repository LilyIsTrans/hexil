use std::sync::Arc;

use super::Renderer;
use super::RendererError;

use super::vert::CanvasSettings;
use tracing::instrument;
use vk::descriptor_set::allocator::DescriptorSetAllocator;
use vulkano as vk;

pub(crate) struct CanvasBuffersManager {
    pub(crate) canvas_settings_host: vk::buffer::Subbuffer<CanvasSettings>,
    pub(crate) canvas_indices_host: vk::buffer::Subbuffer<[u32]>,
    pub(crate) canvas_settings_device: vk::buffer::Subbuffer<CanvasSettings>,
    pub(crate) canvas_indices_device: vk::buffer::Subbuffer<[u32]>,
    pub(crate) descriptors: Option<Arc<vk::descriptor_set::PersistentDescriptorSet>>,
}

impl CanvasBuffersManager {
    #[instrument(skip_all, err)]
    pub fn new(
        renderer: &Renderer,
        width: u32,
        height: u32,
        palette_size: u64,
    ) -> Result<Self, RendererError> {
        let canvas_settings_host = vk::buffer::Buffer::new_unsized::<CanvasSettings>(
            renderer.allocator.clone(),
            vk::buffer::BufferCreateInfo {
                usage: vk::buffer::BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            vk::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vk::memory::allocator::MemoryTypeFilter::HOST_RANDOM_ACCESS
                    | vk::memory::allocator::MemoryTypeFilter::PREFER_HOST,
                ..Default::default()
            },
            palette_size,
        )?;
        let canvas_settings_device = vk::buffer::Buffer::new_unsized::<CanvasSettings>(
            renderer.allocator.clone(),
            vk::buffer::BufferCreateInfo {
                usage: vk::buffer::BufferUsage::TRANSFER_DST
                    | vk::buffer::BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            vk::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            palette_size,
        )?;
        let canvas_indices_host = vk::buffer::Buffer::new_unsized::<[u32]>(
            renderer.allocator.clone(),
            vk::buffer::BufferCreateInfo {
                usage: vk::buffer::BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            vk::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vk::memory::allocator::MemoryTypeFilter::HOST_RANDOM_ACCESS
                    | vk::memory::allocator::MemoryTypeFilter::PREFER_HOST,
                ..Default::default()
            },
            (width as u64) * (height as u64),
        )?;
        let canvas_indices_device = vk::buffer::Buffer::new_unsized::<[u32]>(
            renderer.allocator.clone(),
            vk::buffer::BufferCreateInfo {
                usage: vk::buffer::BufferUsage::TRANSFER_DST
                    | vk::buffer::BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            vk::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            (width as u64) * (height as u64),
        )?;

        {
            let mut guard = canvas_settings_host.write()?;
            guard.WIDTH = width;
            guard.HEIGHT = height.into();
        }
        let mut output = Self {
            canvas_settings_host,
            canvas_indices_host,
            canvas_settings_device,
            canvas_indices_device,
            descriptors: None,
        };
        let descriptors = output.rebuild_descriptors(renderer);
        output.descriptors = Some(descriptors?);
        Ok(output)
    }

    #[instrument(skip_all, err)]
    pub fn rebuild_descriptors(
        &mut self,
        renderer: &Renderer,
    ) -> Result<Arc<vk::descriptor_set::PersistentDescriptorSet>, RendererError> {
        let settings_buffer_info = vk::descriptor_set::DescriptorBufferInfo {
            buffer: self.canvas_settings_device.as_bytes().clone(),
            range: 0..self.canvas_settings_device.size(),
        };
        let settings = vk::descriptor_set::DescriptorBindingResources::Buffer(
            [Some(settings_buffer_info)].into(),
        );
        let indices_buffer_info = vk::descriptor_set::DescriptorBufferInfo {
            buffer: self.canvas_indices_device.as_bytes().clone(),
            range: 0..self.canvas_indices_device.size(),
        };
        let indices = vk::descriptor_set::DescriptorBindingResources::Buffer(
            [Some(indices_buffer_info)].into(),
        );
        let layout = vk::descriptor_set::layout::DescriptorSetLayoutBinding {
            binding_flags: vk::descriptor_set::layout::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            descriptor_count: 1,
            stages: vk::shader::ShaderStages::VERTEX,
            ..vk::descriptor_set::layout::DescriptorSetLayoutBinding::descriptor_type(
                vk::descriptor_set::layout::DescriptorType::UniformBuffer,
            )
        };
        let layout = vk::descriptor_set::layout::DescriptorSetLayoutCreateInfo {
            flags:
                vk::descriptor_set::layout::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL,
            bindings: [(0u32, layout.clone()), (1u32, layout)].into(),
            ..Default::default()
        };
        let layout = vk::descriptor_set::layout::DescriptorSetLayout::new(
            renderer.logical_device.clone(),
            layout,
        )?;

        let write_settings =
            vk::descriptor_set::WriteDescriptorSet::buffer(0, self.canvas_settings_device.clone());
        let write_indices =
            vk::descriptor_set::WriteDescriptorSet::buffer(1, self.canvas_indices_device.clone());

        let set = vk::descriptor_set::PersistentDescriptorSet::new(
            &renderer.descriptor_allocator,
            layout,
            [write_settings, write_indices],
            None,
        )?;

        Ok(set)
    }
}
