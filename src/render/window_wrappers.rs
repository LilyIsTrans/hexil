mod pipeline_wrapper;
use try_log::try_or_err;
use vulkano as vk;

use vk::buffer::Subbuffer;

use super::RendererError;

use super::hex_vert;
use super::tile_frag;

use super::framebuffer::make_framebuffers;

use super::types::Position;

use super::Renderer;

use vk::render_pass::Framebuffer;

use std::sync::Arc;

use vk::pipeline::graphics::viewport::Viewport;

pub(super) struct SwapchainWrapper {
    pub(super) swapchain: Arc<vk::swapchain::Swapchain>,
    pub(super) swapchain_images: Vec<Arc<vk::image::Image>>,
    pub(super) render_pass: Arc<vk::render_pass::RenderPass>,
    pub(super) pipeline: pipeline_wrapper::PipelineWrapper,
}

impl SwapchainWrapper {
    pub fn new(
        renderer: &Renderer,
        size: [u32; 2],
        vert: Arc<vk::shader::ShaderModule>,
        frag: Arc<vk::shader::ShaderModule>,
        vertex_buffer: Subbuffer<[Position]>,
    ) -> Result<Option<SwapchainWrapper>, RendererError> {
        let _guard = tracing::info_span!("SwapchainWrapper::new").entered();
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: size.map(|f| f as f32),
            depth_range: 0.0..=1.0,
        };
        let (swapchain, swapchain_images): (
            Arc<vk::swapchain::Swapchain>,
            Vec<Arc<vk::image::Image>>,
        ) = match renderer.make_swapchain(None, size) {
            Ok(Some(it)) => it,
            Ok(None) => return Ok(None),
            Err(err) => return Err(err),
        };
        let render_pass: Arc<vk::render_pass::RenderPass> =
            renderer.make_renderpass(swapchain.clone())?;
        let framebuffers: Vec<Arc<Framebuffer>> =
            make_framebuffers(&swapchain_images, render_pass.clone())?;

        let pipeline = pipeline_wrapper::PipelineWrapper::new(
            &renderer,
            vert,
            frag,
            vertex_buffer,
            &render_pass,
            viewport,
            &framebuffers,
        )?;

        Ok(Some(Self {
            swapchain,
            swapchain_images,
            render_pass,
            pipeline,
        }))
    }

    pub fn rebuild(
        self,
        renderer: &Renderer,
        size: [u32; 2],
    ) -> Result<Option<SwapchainWrapper>, RendererError> {
        let _guard = tracing::info_span!("SwapchainWrapper::rebuild").entered();
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: size.map(|f| f as f32),
            depth_range: 0.0..=1.0,
        };
        let old_format = self.swapchain.image_format();
        let (swapchain, swapchain_images): (
            Arc<vk::swapchain::Swapchain>,
            Vec<Arc<vk::image::Image>>,
        ) = match renderer.make_swapchain(Some((self.swapchain, self.swapchain_images)), size) {
            Ok(Some(it)) => it,
            Ok(None) => return Ok(None),
            Err(err) => return Err(err),
        };
        let render_pass: Arc<vk::render_pass::RenderPass> =
            if swapchain.image_format() != old_format {
                renderer.make_renderpass(swapchain.clone())?
            } else {
                self.render_pass
            };
        let framebuffers: Vec<Arc<Framebuffer>> =
            make_framebuffers(&swapchain_images, render_pass.clone())?;

        let pipeline = self.pipeline.rebuild(&renderer, viewport, &framebuffers)?;
        Ok(Some(Self {
            swapchain,
            swapchain_images,
            render_pass,
            pipeline,
        }))
    }
}

const SQUARE: [Position; 6] = [
    Position {
        position: [0.0, 0.0],
    },
    Position {
        position: [-1.0, -1.0],
    },
    Position {
        position: [1.0, -1.0],
    },
    Position {
        position: [1.0, 1.0],
    },
    Position {
        position: [-1.0, 1.0],
    },
    Position {
        position: [-1.0, -1.0],
    },
];

#[allow(dead_code)]
const HEXAGON: [Position; 8] = [
    Position {
        position: [0.0, 0.0],
    },
    Position {
        position: [-0.5, -1.0],
    },
    Position {
        position: [0.5, -1.0],
    },
    Position {
        position: [1.0, 0.0],
    },
    Position {
        position: [0.5, 1.0],
    },
    Position {
        position: [-0.5, 1.0],
    },
    Position {
        position: [-1.0, 0.0],
    },
    Position {
        position: [-0.5, -1.0],
    },
];

impl SwapchainWrapper {
    pub(super) fn make_canvas_swapchain(
        renderer: &Renderer,
        size: [u32; 2],
    ) -> Result<Option<SwapchainWrapper>, RendererError> {
        let _guard = tracing::info_span!("SwapchainWrapper::make_canvas_swapchain").entered();
        let vertex_buffer = vk::buffer::Buffer::from_iter(
            renderer.allocator.clone(),
            vk::buffer::BufferCreateInfo {
                usage: vk::buffer::BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            vk::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vk::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vk::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            SQUARE,
        )?;
        use tracing::error;
        let vert: Arc<vk::shader::ShaderModule> =
            match hex_vert::load(renderer.logical_device.clone()) {
                Ok(expr) => expr,
                Err(vk::Validated::ValidationError(err)) => {
                    error!("Try expression failed with: {}", err);
                    return Err(err.into());
                }
                Err(vk::Validated::Error(err)) => {
                    error!("Try expression failed with: {}", err);
                    return Err(err.into());
                }
            };
        let frag: Arc<vk::shader::ShaderModule> =
            try_or_err!(tile_frag::load(renderer.logical_device.clone()));

        Ok(SwapchainWrapper::new(
            &renderer,
            size,
            vert.clone(),
            frag.clone(),
            vertex_buffer,
        )?)
    }
}
