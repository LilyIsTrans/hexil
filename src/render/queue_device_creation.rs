use std::sync::Arc;

use super::renderer_error;
use tracing::error;
use tracing::instrument;
use try_log::log_tries;
use vulkano as vk;

use tracing::warn;

use super::Renderer;

impl Renderer {
    /// On success, returns a tuple `(device, transfer_queue, graphics_queue)`.
    // #[instrument(skip_all)]
    // #[log_tries(tracing::error)]
    pub(crate) fn get_queues_and_device(
        physical_device: Arc<vk::device::physical::PhysicalDevice>,
    ) -> Result<
        (
            Arc<vk::device::Device>,
            Arc<vk::device::Queue>,
            Arc<vk::device::Queue>,
        ),
        renderer_error::RendererError,
    > {
        let graphics_queue_flags = vk::device::QueueFlags::GRAPHICS;
        let transfer_queue_flags = vk::device::QueueFlags::TRANSFER;
        let both = graphics_queue_flags.union(transfer_queue_flags);
        let queue_family_indices = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .filter(|(_, p)| p.queue_flags.intersects(both));
        let graphics_queue_families: Vec<usize> = queue_family_indices
            .clone()
            .filter(|(_, p)| p.queue_flags.contains(graphics_queue_flags))
            .map(|(i, _)| i)
            .collect();
        let transfer_queue_families: Vec<usize> = queue_family_indices
            .filter(|(_, p)| p.queue_flags.contains(transfer_queue_flags))
            .map(|(i, _)| i)
            .collect();

        let (both, graphics_only): (Vec<usize>, Vec<usize>) = graphics_queue_families
            .iter()
            .partition(|i| transfer_queue_families.contains(i));
        let transfer_only: Vec<usize> = transfer_queue_families
            .iter()
            .filter(|i| !graphics_only.contains(i))
            .copied()
            .collect();
        // Selects a graphics queue family and a transfer queue family.
        // If possible, it will select different queue families.
        let (graphics_family, transfer_family) =
            match (both.len(), graphics_only.len(), transfer_only.len()) {
                (0, 0, _) => {
                    error!("No graphics queues!");
                    return Err(renderer_error::RendererError::NoGraphicsQueues);
                }
                (0, _, 0) => {
                    error!("No transfer queues!");
                    return Err(renderer_error::RendererError::NoTransferQueues);
                }
                (1, 0, 0) => {
                    warn!("Only one queue available, performance may be affected.");
                    let q = both
                        .first()
                        .expect("We just confirmed that both has exactly 1 element.");
                    (*q, *q)
                }
                (_, 0, 0) => (both[0], both[1]),
                (_, 0, _) => (both[0], transfer_only[0]),
                (_, _, 0) => (graphics_only[0], both[0]),
                (_, _, _) => (graphics_only[0], transfer_only[0]),
            };

        let mut queues = vec![0.5];
        if graphics_family == transfer_family {
            queues.push(0.5);
        }
        let graphics_queue_create_info = vk::device::QueueCreateInfo {
            queue_family_index: graphics_family
                .try_into()
                .expect("I got this index from this device. It better be able to take it back."),
            queues,
            ..Default::default()
        };

        let transfer_queue_create_info = vk::device::QueueCreateInfo {
            queue_family_index: transfer_family
                .try_into()
                .expect("I got this index from this device. It better be able to take it back."),
            ..Default::default()
        };

        let mut queue_create_infos = Vec::<vk::device::QueueCreateInfo>::with_capacity(2);

        queue_create_infos.push(graphics_queue_create_info);
        if graphics_family != transfer_family {
            queue_create_infos.push(transfer_queue_create_info);
        };

        // let needed_extensions = vk::device::DeviceExtensions {
        //     khr_buffer_device_address: todo!(),
        //     khr_copy_commands2: todo!(),
        //     khr_create_renderpass2: todo!(),
        //     khr_dedicated_allocation: todo!(),
        //     khr_deferred_host_operations: todo!(),
        //     khr_depth_stencil_resolve: todo!(),
        //     khr_descriptor_update_template: todo!(),
        //     khr_device_group: todo!(),
        //     khr_display_swapchain: todo!(),
        //     khr_draw_indirect_count: todo!(),
        //     khr_driver_properties: todo!(),
        //     khr_dynamic_rendering: todo!(),
        //     khr_external_fence: todo!(),
        //     khr_external_fence_fd: todo!(),
        //     khr_external_fence_win32: todo!(),
        //     khr_external_memory: todo!(),
        //     khr_external_memory_fd: todo!(),
        //     khr_external_memory_win32: todo!(),
        //     khr_external_semaphore: todo!(),
        //     khr_external_semaphore_fd: todo!(),
        //     khr_external_semaphore_win32: todo!(),
        //     khr_format_feature_flags2: todo!(),
        //     khr_fragment_shader_barycentric: todo!(),
        //     khr_fragment_shading_rate: todo!(),
        //     khr_get_memory_requirements2: todo!(),
        //     khr_global_priority: todo!(),
        //     khr_image_format_list: todo!(),
        //     khr_imageless_framebuffer: todo!(),
        //     khr_incremental_present: todo!(),
        //     khr_maintenance1: todo!(),
        //     khr_maintenance2: todo!(),
        //     khr_maintenance3: todo!(),
        //     khr_maintenance4: todo!(),
        //     khr_map_memory2: todo!(),
        //     khr_multiview: todo!(),
        //     khr_performance_query: todo!(),
        //     khr_pipeline_executable_properties: todo!(),
        //     khr_pipeline_library: todo!(),
        //     khr_portability_subset: todo!(),
        //     khr_present_id: todo!(),
        //     khr_present_wait: todo!(),
        //     khr_push_descriptor: todo!(),
        //     khr_ray_query: todo!(),
        //     khr_ray_tracing_maintenance1: todo!(),
        //     khr_ray_tracing_pipeline: todo!(),
        //     khr_ray_tracing_position_fetch: todo!(),
        //     khr_relaxed_block_layout: todo!(),
        //     khr_sampler_mirror_clamp_to_edge: todo!(),
        //     khr_sampler_ycbcr_conversion: todo!(),
        //     khr_separate_depth_stencil_layouts: todo!(),
        //     khr_shader_atomic_int64: todo!(),
        //     khr_shader_clock: todo!(),
        //     khr_shader_draw_parameters: todo!(),
        //     khr_shader_float16_int8: todo!(),
        //     khr_shader_float_controls: todo!(),
        //     khr_shader_integer_dot_product: todo!(),
        //     khr_shader_non_semantic_info: todo!(),
        //     khr_shader_subgroup_extended_types: todo!(),
        //     khr_shader_subgroup_uniform_control_flow: todo!(),
        //     khr_shader_terminate_invocation: todo!(),
        //     khr_shared_presentable_image: todo!(),
        //     khr_spirv_1_4: todo!(),
        //     khr_storage_buffer_storage_class: todo!(),
        //     khr_swapchain: todo!(),
        //     khr_swapchain_mutable_format: todo!(),
        //     khr_synchronization2: todo!(),
        //     khr_timeline_semaphore: todo!(),
        //     khr_uniform_buffer_standard_layout: todo!(),
        //     khr_variable_pointers: todo!(),
        //     khr_video_decode_h264: todo!(),
        //     khr_video_decode_h265: todo!(),
        //     khr_video_decode_queue: todo!(),
        //     khr_video_encode_queue: todo!(),
        //     khr_video_queue: todo!(),
        //     khr_vulkan_memory_model: todo!(),
        //     khr_win32_keyed_mutex: todo!(),
        //     khr_workgroup_memory_explicit_layout: todo!(),
        //     khr_zero_initialize_workgroup_memory: todo!(),
        //     ext_4444_formats: todo!(),
        //     ext_astc_decode_mode: todo!(),
        //     ext_attachment_feedback_loop_dynamic_state: todo!(),
        //     ext_attachment_feedback_loop_layout: todo!(),
        //     ext_blend_operation_advanced: todo!(),
        //     ext_border_color_swizzle: todo!(),
        //     ext_buffer_device_address: todo!(),
        //     ext_calibrated_timestamps: todo!(),
        //     ext_color_write_enable: todo!(),
        //     ext_conditional_rendering: todo!(),
        //     ext_conservative_rasterization: todo!(),
        //     ext_custom_border_color: todo!(),
        //     ext_debug_marker: todo!(),
        //     ext_depth_clamp_zero_one: todo!(),
        //     ext_depth_clip_control: todo!(),
        //     ext_depth_clip_enable: todo!(),
        //     ext_depth_range_unrestricted: todo!(),
        //     ext_descriptor_buffer: todo!(),
        //     ext_descriptor_indexing: todo!(),
        //     ext_device_address_binding_report: todo!(),
        //     ext_device_fault: todo!(),
        //     ext_device_memory_report: todo!(),
        //     ext_discard_rectangles: todo!(),
        //     ext_display_control: todo!(),
        //     ext_dynamic_rendering_unused_attachments: todo!(),
        //     ext_extended_dynamic_state: todo!(),
        //     ext_extended_dynamic_state2: todo!(),
        //     ext_extended_dynamic_state3: todo!(),
        //     ext_external_memory_dma_buf: todo!(),
        //     ext_external_memory_host: todo!(),
        //     ext_filter_cubic: todo!(),
        //     ext_fragment_density_map: todo!(),
        //     ext_fragment_density_map2: todo!(),
        //     ext_fragment_shader_interlock: todo!(),
        //     ext_full_screen_exclusive: todo!(),
        //     ext_global_priority: todo!(),
        //     ext_global_priority_query: todo!(),
        //     ext_graphics_pipeline_library: todo!(),
        //     ext_hdr_metadata: todo!(),
        //     ext_host_query_reset: todo!(),
        //     ext_image_2d_view_of_3d: todo!(),
        //     ext_image_compression_control: todo!(),
        //     ext_image_compression_control_swapchain: todo!(),
        //     ext_image_drm_format_modifier: todo!(),
        //     ext_image_robustness: todo!(),
        //     ext_image_sliced_view_of_3d: todo!(),
        //     ext_image_view_min_lod: todo!(),
        //     ext_index_type_uint8: todo!(),
        //     ext_inline_uniform_block: todo!(),
        //     ext_legacy_dithering: todo!(),
        //     ext_line_rasterization: todo!(),
        //     ext_load_store_op_none: todo!(),
        //     ext_memory_budget: todo!(),
        //     ext_memory_priority: todo!(),
        //     ext_mesh_shader: todo!(),
        //     ext_metal_objects: todo!(),
        //     ext_multi_draw: todo!(),
        //     ext_multisampled_render_to_single_sampled: todo!(),
        //     ext_mutable_descriptor_type: todo!(),
        //     ext_non_seamless_cube_map: todo!(),
        //     ext_opacity_micromap: todo!(),
        //     ext_pageable_device_local_memory: todo!(),
        //     ext_pci_bus_info: todo!(),
        //     ext_physical_device_drm: todo!(),
        //     ext_pipeline_creation_cache_control: todo!(),
        //     ext_pipeline_creation_feedback: todo!(),
        //     ext_pipeline_library_group_handles: todo!(),
        //     ext_pipeline_properties: todo!(),
        //     ext_pipeline_protected_access: todo!(),
        //     ext_pipeline_robustness: todo!(),
        //     ext_post_depth_coverage: todo!(),
        //     ext_primitive_topology_list_restart: todo!(),
        //     ext_primitives_generated_query: todo!(),
        //     ext_private_data: todo!(),
        //     ext_provoking_vertex: todo!(),
        //     ext_queue_family_foreign: todo!(),
        //     ext_rasterization_order_attachment_access: todo!(),
        //     ext_rgba10x6_formats: todo!(),
        //     ext_robustness2: todo!(),
        //     ext_sample_locations: todo!(),
        //     ext_sampler_filter_minmax: todo!(),
        //     ext_scalar_block_layout: todo!(),
        //     ext_separate_stencil_usage: todo!(),
        //     ext_shader_atomic_float: todo!(),
        //     ext_shader_atomic_float2: todo!(),
        //     ext_shader_demote_to_helper_invocation: todo!(),
        //     ext_shader_image_atomic_int64: todo!(),
        //     ext_shader_module_identifier: todo!(),
        //     ext_shader_object: todo!(),
        //     ext_shader_stencil_export: todo!(),
        //     ext_shader_subgroup_ballot: todo!(),
        //     ext_shader_subgroup_vote: todo!(),
        //     ext_shader_tile_image: todo!(),
        //     ext_shader_viewport_index_layer: todo!(),
        //     ext_subgroup_size_control: todo!(),
        //     ext_subpass_merge_feedback: todo!(),
        //     ext_swapchain_maintenance1: todo!(),
        //     ext_texel_buffer_alignment: todo!(),
        //     ext_texture_compression_astc_hdr: todo!(),
        //     ext_tooling_info: todo!(),
        //     ext_transform_feedback: todo!(),
        //     ext_validation_cache: todo!(),
        //     ext_vertex_attribute_divisor: todo!(),
        //     ext_vertex_input_dynamic_state: todo!(),
        //     ext_video_encode_h264: todo!(),
        //     ext_video_encode_h265: todo!(),
        //     ext_ycbcr_2plane_444_formats: todo!(),
        //     ext_ycbcr_image_arrays: todo!(),
        //     amd_buffer_marker: todo!(),
        //     amd_device_coherent_memory: todo!(),
        //     amd_display_native_hdr: todo!(),
        //     amd_draw_indirect_count: todo!(),
        //     amd_gcn_shader: todo!(),
        //     amd_gpu_shader_half_float: todo!(),
        //     amd_gpu_shader_int16: todo!(),
        //     amd_memory_overallocation_behavior: todo!(),
        //     amd_mixed_attachment_samples: todo!(),
        //     amd_pipeline_compiler_control: todo!(),
        //     amd_rasterization_order: todo!(),
        //     amd_shader_ballot: todo!(),
        //     amd_shader_core_properties: todo!(),
        //     amd_shader_core_properties2: todo!(),
        //     amd_shader_early_and_late_fragment_tests: todo!(),
        //     amd_shader_explicit_vertex_parameter: todo!(),
        //     amd_shader_fragment_mask: todo!(),
        //     amd_shader_image_load_store_lod: todo!(),
        //     amd_shader_info: todo!(),
        //     amd_shader_trinary_minmax: todo!(),
        //     amd_texture_gather_bias_lod: todo!(),
        //     android_external_memory_android_hardware_buffer: todo!(),
        //     arm_rasterization_order_attachment_access: todo!(),
        //     arm_shader_core_builtins: todo!(),
        //     arm_shader_core_properties: todo!(),
        //     fuchsia_buffer_collection: todo!(),
        //     fuchsia_external_memory: todo!(),
        //     fuchsia_external_semaphore: todo!(),
        //     ggp_frame_token: todo!(),
        //     google_decorate_string: todo!(),
        //     google_display_timing: todo!(),
        //     google_hlsl_functionality1: todo!(),
        //     google_user_type: todo!(),
        //     huawei_cluster_culling_shader: todo!(),
        //     huawei_invocation_mask: todo!(),
        //     huawei_subpass_shading: todo!(),
        //     img_filter_cubic: todo!(),
        //     img_format_pvrtc: todo!(),
        //     intel_performance_query: todo!(),
        //     intel_shader_integer_functions2: todo!(),
        //     nvx_binary_import: todo!(),
        //     nvx_image_view_handle: todo!(),
        //     nvx_multiview_per_view_attributes: todo!(),
        //     nv_acquire_winrt_display: todo!(),
        //     nv_clip_space_w_scaling: todo!(),
        //     nv_compute_shader_derivatives: todo!(),
        //     nv_cooperative_matrix: todo!(),
        //     nv_copy_memory_indirect: todo!(),
        //     nv_corner_sampled_image: todo!(),
        //     nv_coverage_reduction_mode: todo!(),
        //     nv_dedicated_allocation: todo!(),
        //     nv_dedicated_allocation_image_aliasing: todo!(),
        //     nv_device_diagnostic_checkpoints: todo!(),
        //     nv_device_diagnostics_config: todo!(),
        //     nv_device_generated_commands: todo!(),
        //     nv_displacement_micromap: todo!(),
        //     nv_external_memory: todo!(),
        //     nv_external_memory_rdma: todo!(),
        //     nv_external_memory_win32: todo!(),
        //     nv_fill_rectangle: todo!(),
        //     nv_fragment_coverage_to_color: todo!(),
        //     nv_fragment_shader_barycentric: todo!(),
        //     nv_fragment_shading_rate_enums: todo!(),
        //     nv_framebuffer_mixed_samples: todo!(),
        //     nv_geometry_shader_passthrough: todo!(),
        //     nv_glsl_shader: todo!(),
        //     nv_inherited_viewport_scissor: todo!(),
        //     nv_linear_color_attachment: todo!(),
        //     nv_low_latency: todo!(),
        //     nv_memory_decompression: todo!(),
        //     nv_mesh_shader: todo!(),
        //     nv_optical_flow: todo!(),
        //     nv_present_barrier: todo!(),
        //     nv_ray_tracing: todo!(),
        //     nv_ray_tracing_invocation_reorder: todo!(),
        //     nv_ray_tracing_motion_blur: todo!(),
        //     nv_representative_fragment_test: todo!(),
        //     nv_sample_mask_override_coverage: todo!(),
        //     nv_scissor_exclusive: todo!(),
        //     nv_shader_image_footprint: todo!(),
        //     nv_shader_sm_builtins: todo!(),
        //     nv_shader_subgroup_partitioned: todo!(),
        //     nv_shading_rate_image: todo!(),
        //     nv_viewport_array2: todo!(),
        //     nv_viewport_swizzle: todo!(),
        //     nv_win32_keyed_mutex: todo!(),
        //     qcom_fragment_density_map_offset: todo!(),
        //     qcom_image_processing: todo!(),
        //     qcom_multiview_per_view_render_areas: todo!(),
        //     qcom_multiview_per_view_viewports: todo!(),
        //     qcom_render_pass_shader_resolve: todo!(),
        //     qcom_render_pass_store_ops: todo!(),
        //     qcom_render_pass_transform: todo!(),
        //     qcom_rotated_copy_commands: todo!(),
        //     qcom_tile_properties: todo!(),
        //     sec_amigo_profiling: todo!(),
        //     valve_descriptor_set_host_mapping: todo!(),
        //     valve_mutable_descriptor_type: todo!(),
        //     ..Default::default()
        // };

        let logical_device = vk::device::DeviceCreateInfo {
            queue_create_infos,
            enabled_extensions: physical_device
                .supported_extensions()
                .intersection(&super::consts::ALL_KHR_DEVICE_EXTENSIONS.clone()),
            // .union(&needed_extensions),
            // enabled_features: Features {
            //     bresenham_lines: true,
            //     ..Default::default()
            // },
            ..Default::default()
        };

        let (device, queues) = vk::device::Device::new(physical_device.clone(), logical_device)?;

        let queues = queues.collect::<Arc<[_]>>();
        let graphics_queue: Arc<vk::device::Queue> = queues
            .iter()
            .find(|q| {
                graphics_family
                    == q.queue_family_index()
                        .try_into()
                        .expect("I sure hope u32 fits into usize.")
            })
            .expect("If it didn't exist, we'd have returned an error a few lines ago.")
            .clone();
        let transfer_queue: Arc<vk::device::Queue> = queues
            .iter()
            .find(|q| {
                transfer_family
                    == q.queue_family_index()
                        .try_into()
                        .expect("I sure hope u32 fits into usize.")
                    && q.id_within_family() != graphics_queue.id_within_family()
            })
            .expect("If it didn't exist, we'd have returned an error a few lines ago.")
            .clone();

        Ok((device, transfer_queue, graphics_queue))
    }
}
