use vk::buffer as vbuf;
use vulkano as vk;

use super::LayerV1Canvas;

use std::sync::Arc;

use super::LayerV1CanvasHost;

impl LayerV1CanvasHost {
    pub fn convert(
        self,
        alloc: Arc<dyn vk::memory::allocator::MemoryAllocator>,
    ) -> Result<LayerV1Canvas, crate::render::RendererError> {
        Ok(match self {
            LayerV1CanvasHost::Alpha(buf) => LayerV1Canvas::Alpha(vk::buffer::Buffer::from_iter(
                alloc,
                vbuf::BufferCreateInfo {
                    usage: vk::buffer::BufferUsage::TRANSFER_SRC,
                    ..Default::default()
                },
                vk::memory::allocator::AllocationCreateInfo {
                    memory_type_filter:
                        vk::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE
                            | vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
                buf.into_iter(),
            )?),
            LayerV1CanvasHost::BaseColor { palette, canvas } => LayerV1Canvas::BaseColor {
                palette: vk::buffer::Buffer::from_iter(
                    alloc.clone(),
                    vbuf::BufferCreateInfo {
                        usage: vk::buffer::BufferUsage::TRANSFER_SRC,
                        ..Default::default()
                    },
                    vk::memory::allocator::AllocationCreateInfo {
                        memory_type_filter:
                            vk::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE
                                | vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
                        ..Default::default()
                    },
                    palette.into_iter(),
                )?,
                canvas: vk::buffer::Buffer::from_iter(
                    alloc,
                    vbuf::BufferCreateInfo {
                        usage: vk::buffer::BufferUsage::TRANSFER_SRC,
                        ..Default::default()
                    },
                    vk::memory::allocator::AllocationCreateInfo {
                        memory_type_filter:
                            vk::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE
                                | vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
                        ..Default::default()
                    },
                    canvas.into_iter(),
                )?,
            },
            LayerV1CanvasHost::Shading(buf) => {
                LayerV1Canvas::Shading(vk::buffer::Buffer::from_iter(
                    alloc,
                    vbuf::BufferCreateInfo {
                        usage: vk::buffer::BufferUsage::TRANSFER_SRC,
                        ..Default::default()
                    },
                    vk::memory::allocator::AllocationCreateInfo {
                        memory_type_filter:
                            vk::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE
                                | vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
                        ..Default::default()
                    },
                    buf.into_iter(),
                )?)
            }
        })
    }
}
