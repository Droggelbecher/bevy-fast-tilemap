use bevy::{
    ecs::system::SystemState,
    prelude::*,
    render::{
        render_resource::*,
        renderer::RenderDevice,
        texture::{BevyDefault, DefaultImageSampler},
        view::ViewTarget,
    },
    sprite::{Mesh2dPipeline, Mesh2dPipelineKey},
};

use crate::shader::SHADER_HANDLE;

#[derive(Resource)]
pub struct MapPipeline {
    /// this pipeline wraps the standard [`Mesh2dPipeline`]
    pub mesh2d_pipeline: Mesh2dPipeline,
    pub map_layout: BindGroupLayout,
}

impl FromWorld for MapPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut system_state: SystemState<(Res<RenderDevice>, Res<DefaultImageSampler>)> =
            SystemState::new(world);
        let (render_device, _default_sampler) = system_state.get_mut(world);

        // The layout of bind group 1, i.e. types from shader point of view
        // bevy calls this "layout pipeline layout"

        let map_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::R16Uint,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("fast_tilemap_layout"),
        });

        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
            map_layout,
        } // Self
    } // from_world()
}

impl SpecializedRenderPipeline for MapPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let vertex_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Vertex,
            vec![
                // Position
                VertexFormat::Float32x3,
                // Normal
                VertexFormat::Float32x3,
                // UV
                VertexFormat::Float32x2,
            ],
        );

        let format = match key.contains(Mesh2dPipelineKey::HDR) {
            true => ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![vertex_layout],
            },

            fragment: Some(FragmentState {
                shader: SHADER_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),

            // Use the two standard uniforms for 2d meshes
            layout: vec![
                // Bind group 0 is the view uniform
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 is our stuff
                self.map_layout.clone(),
                // Bind group 2 is the mesh uniform
                self.mesh2d_pipeline.mesh_layout.clone(),
            ],

            push_constant_ranges: Vec::new(),
            depth_stencil: None,
            label: Some("fast_tilemap_pipeline".into()),

            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },

            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        } // RenderPipelineDescriptor
    } // specialize()
} // impl SpeceializedRenderPipeline
