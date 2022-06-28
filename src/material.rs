use crate::map::MapLayerMaterial;
use bevy::{
    reflect::TypeUuid,
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    math::vec2,
    prelude::*,
    render::{
        mesh::InnerMeshVertexBufferLayout,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssets},
        render_resource::{
            std140::{AsStd140, Std140},
            *,
        },
        renderer::RenderDevice,
    },
    sprite::{Material2d, Material2dPipeline},
    utils::{FixedState, Hashed},
};

pub const TILEMAP_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 12251966238699670935);

pub const TILEMAP_SHADER: &str = include_str!("tilemap_shader.wgsl");

#[derive(Clone)]
pub struct GpuMapLayerMaterial {
    bind_group: BindGroup,
}

impl RenderAsset for MapLayerMaterial {
    type ExtractedAsset = MapLayerMaterial;
    type PreparedAsset = GpuMapLayerMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<RenderAssets<Image>>,
        SRes<Material2dPipeline<Self>>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, images, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let map_texture = match images.get(&extracted_asset.map_texture) {
            Some(texture) => texture,
            None => {
                return Err(PrepareAssetError::RetryNextUpdate(extracted_asset));
            }
        };

        let tint_texture = match images.get(&extracted_asset.tint_texture) {
            Some(texture) => texture,
            None => {
                return Err(PrepareAssetError::RetryNextUpdate(extracted_asset));
            }
        };

        let tiles_texture = match images.get(&extracted_asset.tiles_texture) {
            Some(texture) => texture,
            None => {
                return Err(PrepareAssetError::RetryNextUpdate(extracted_asset));
            }
        };

        let tilemap_size =
            vec2(tiles_texture.size.width, tiles_texture.size.height) / extracted_asset.tile_size;

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: tilemap_size.as_std140().as_bytes(),
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&map_texture.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&tint_texture.texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&tiles_texture.texture_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&tiles_texture.sampler),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: buffer.as_entire_binding(),
                },
            ],
            label: None,
            layout: &material_pipeline.material2d_layout,
        });

        Ok(GpuMapLayerMaterial { bind_group })
    } // fn prepare_asset
} // impl RenderAsset for TileMapMaterial

impl Material2d for MapLayerMaterial {
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &Hashed<InnerMeshVertexBufferLayout, FixedState>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "vertex".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "fragment".into();
        Ok(())
    }

    fn vertex_shader(_asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(TILEMAP_SHADER_HANDLE.typed::<Shader>())
    }

    fn fragment_shader(_asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(TILEMAP_SHADER_HANDLE.typed::<Shader>())
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        let l = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                // Map texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::R16Uint,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },

                // Tint texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba8Uint,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },


                // Tiles texture
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(Vec2::std140_size_static() as u64),
                    },
                    count: None,
                },
            ],
            label: None,
        });
        l
    }
} // impl Material2d for TileMapMaterial
