use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, AsBindGroupError, AsBindGroupError::RetryNextUpdate,
            BindGroupDescriptor, BindGroupEntry, BindGroupLayout, OwnedBindingResource,
            PreparedBindGroup, encase::UniformBuffer, BufferInitDescriptor, BufferUsages, ShaderType
        },
        renderer::RenderDevice,
        texture::FallbackImage,
    },
    sprite::{
         Mesh2dHandle, Mesh2dUniform
    },
};

use crate::{
    extract::ExtractedMap,
    pipeline::FastTileMapPipeline
};

#[derive(Component)]
pub struct PreparedMap {
    pub bind_group: PreparedBindGroup<()>,
}

/// Prepare data for GPU
/// More precisely, generated `PreparedBindGroup`s and
/// store them somewhere in the render world (eg. a resource that holds them such as
/// RenderMaterials2d)
///
/// This is a system in the render app.
pub fn prepare_fast_tilemap(
    extracted_maps: Query<(Entity, &Mesh2dHandle, &Mesh2dUniform, &ExtractedMap)>,
    render_device: Res<RenderDevice>,
    images: Res<RenderAssets<Image>>,
    fallback_image: Res<FallbackImage>,
    pipeline: Res<FastTileMapPipeline>,
    mut commands: Commands,
) {
    let prepared_maps: Vec<(Entity, (Mesh2dHandle, Mesh2dUniform, PreparedMap))> = extracted_maps
        .iter()
        .map(|(entity, mesh_handle, mesh_uniform, extracted_map)| {
            let prepared_map = PreparedMap {
                bind_group: match extracted_map.as_bind_group(
                    &pipeline.map_layout,
                    &render_device,
                    &images,
                    &fallback_image,
                ) {
                    Ok(x) => x,
                    // TODO: in this case we should queue them to try again next frame!
                    // see bevy_sprite/src/mesh2d/material.rs
                    Err(_) => panic!("Couldnt extract bind group"),
                },
            };

            (
                entity,
                (mesh_handle.clone(), mesh_uniform.clone(), prepared_map),
            )
        })
        .collect();

    commands.insert_or_spawn_batch(prepared_maps);
}

impl AsBindGroup for ExtractedMap {
    type Data = ();

    fn bind_group_layout(_render_device: &RenderDevice) -> BindGroupLayout
    where
        Self: Sized,
    {
        // Seems this is not actually called?!
        todo!()
    }

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<Image>,
        _fallback_image: &FallbackImage,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        // See /home/henning/repos/bevy/crates/bevy_render/macros/src/as_bind_group.rs:262
        let tiles_texture_size = images.get(&self.tiles_texture)
            .ok_or_else(|| RetryNextUpdate)?
            .size;

        #[derive(ShaderType)]
        struct MapData {
            tilemap_tiles: Vec2,
            tile_size: Vec2
        }

        let map_data = MapData {
            tilemap_tiles: tiles_texture_size / self.tile_size,
            tile_size: self.tile_size
        };

        let mut map_data_buffer = UniformBuffer::new(Vec::new());
        map_data_buffer.write(&map_data).unwrap();

        let bindings = vec![
            //@group(1) @binding(0)
            //var map_texture: texture_storage_2d<r16uint, read>;
            OwnedBindingResource::TextureView({
                images
                    .get(&self.map_texture)
                    .ok_or_else(|| RetryNextUpdate)?
                    .texture_view
                    .clone()
            }),
            //@group(1) @binding(1)
            //var tiles_texture: texture_2d<f32>;
            OwnedBindingResource::TextureView({
                images
                    .get(&self.tiles_texture)
                    .ok_or_else(|| RetryNextUpdate)?
                    .texture_view
                    .clone()
            }),

            //@group(1) @binding(2)
            //var tiles_sampler: sampler;
            OwnedBindingResource::Sampler({
                images
                    .get(&self.tiles_texture)
                    .ok_or_else(|| RetryNextUpdate)?
                    .sampler
                    .clone()
            }),

            //@group(1) @binding(3)
            //var<uniform> map: Map;
            //struct Map {
            //tilemap_size: vec2<f32>,
            //};
            OwnedBindingResource::Buffer(
                render_device.create_buffer_with_data(
                    &BufferInitDescriptor {
                        label: Some("Map"),
                        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                        contents: map_data_buffer.as_ref(),
                    }
                )
            ),
        ];

        let bind_group = {
            let descriptor = BindGroupDescriptor {
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: bindings[0].get_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: bindings[1].get_binding(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: bindings[2].get_binding(),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: bindings[3].get_binding(),
                    },
                ],
                label: None,
                layout: &layout,
            };
            render_device.create_bind_group(&descriptor)
        };

        Ok(PreparedBindGroup::<Self::Data> {
            bindings,
            bind_group,
            data: (),
        })
    }

}
