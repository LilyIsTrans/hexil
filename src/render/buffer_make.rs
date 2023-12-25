use vulkano as vk;

use super::{renderer_error, Renderer};

use vk::buffer::Subbuffer;

use std::sync::Arc;

use tracing::instrument;

impl Renderer {
    #[instrument(skip(allocator))]
    pub(crate) fn make_buffer<T: vk::buffer::BufferContents>(
        allocator: Arc<impl vk::memory::allocator::MemoryAllocator>,
        usage: vk::buffer::BufferUsage,
        memory_type: vk::memory::allocator::MemoryTypeFilter,
        size: usize,
        len: usize,
    ) -> Result<vk::buffer::Subbuffer<[T]>, renderer_error::RendererError> {
        let raw_square_buf_info = vk::buffer::BufferCreateInfo {
            flags: vk::buffer::BufferCreateFlags::empty(),
            size: (size)
                .try_into()
                .expect("I sure hope the size of a few floats and u16s fits in a u64"),
            usage,
            ..Default::default()
        };
        let mut square_buf_info = raw_square_buf_info.clone();
        square_buf_info.size = 0;
        let square_buf_info = square_buf_info;
        let square_alloc_info = vk::memory::allocator::AllocationCreateInfo {
            memory_type_filter: memory_type,
            ..Default::default()
        };
        let square_buf = vk::buffer::Buffer::new_unsized(
            allocator.clone(),
            square_buf_info,
            square_alloc_info,
            len.try_into().unwrap(),
        );
        Ok(square_buf?)
    }

    #[instrument(skip_all)]
    pub(crate) fn make_staging_buffer<T: vk::buffer::BufferContents>(
        allocator: Arc<impl vk::memory::allocator::MemoryAllocator>,
        iter: impl Iterator<Item = T> + std::iter::ExactSizeIterator,
    ) -> Result<vk::buffer::Subbuffer<[T]>, renderer_error::RendererError> {
        Self::make_buffer(
            allocator,
            vk::buffer::BufferUsage::TRANSFER_SRC,
            vk::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            std::mem::size_of::<T>() * iter.len(),
            iter.len(),
        )
    }
}
