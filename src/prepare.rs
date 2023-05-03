use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, AsBindGroupError, AsBindGroupError::RetryNextUpdate,
            BindGroupDescriptor, BindGroupEntry, BindGroupLayout, OwnedBindingResource,
            PreparedBindGroup, encase::UniformBuffer, BufferInitDescriptor, BufferUsages
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
    pipeline::MapPipeline
};

#[derive(Component)]
pub struct PreparedMap {
    pub bind_group: PreparedBindGroup<()>,
}

/// Prepare data for GPU
/// More precisely, generate `PreparedBindGroup`s and
/// store them somewhere in the render world (eg. a resource that holds them such as
/// RenderMaterials2d)
///
/// This is a system in the render app.
pub fn prepare_fast_tilemap(
    extracted_maps: Query<(Entity, &Mesh2dHandle, &Mesh2dUniform, &ExtractedMap)>,
    render_device: Res<RenderDevice>,
    images: Res<RenderAssets<Image>>,
    fallback_image: Res<FallbackImage>,
    pipeline: Res<MapPipeline>,
    mut commands: Commands,
) {
    let mut prepared_maps = Vec::new();

    for (entity, mesh_handle, mesh_uniform, extracted_map) in extracted_maps.iter() {
        let prepared_map = PreparedMap {
            bind_group: match extracted_map.as_bind_group(
                &pipeline.map_layout,
                &render_device,
                &images,
                &fallback_image,
            ) {
                Ok(x) => x,
                Err(AsBindGroupError::RetryNextUpdate) => {
                    // This implies the map data texture was not yet available in the asset server.
                    //
                    // There is no point in requeueing:
                    // If the map is still there next frame it will be covered by the query then
                    // and if not, it shouldnt be rendered anymore anyways.
                    continue
                }
            },
        };

        prepared_maps.push((
            entity,
            (mesh_handle.clone(), mesh_uniform.clone(), prepared_map),
        ));
    }

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
        let map_uniform = &self.0.map_uniform;

        let mut map_data_buffer = UniformBuffer::new(Vec::new());
        map_data_buffer.write(&map_uniform).unwrap();

        let bindings = vec![
            //@group(1) @binding(0)
            //var map_texture: texture_storage_2d<r16uint, read>;
            OwnedBindingResource::TextureView({
                images
                    .get(&self.0.map_texture)
                    .ok_or_else(|| RetryNextUpdate)?
                    .texture_view
                    .clone()
            }),
            //@group(1) @binding(1)
            //var atlas_texture: texture_2d<f32>;
            OwnedBindingResource::TextureView({
                images
                    .get(&self.0.atlas_texture)
                    .ok_or_else(|| RetryNextUpdate)?
                    .texture_view
                    .clone()
            }),

            //@group(1) @binding(2)
            //var tiles_sampler: sampler;
            OwnedBindingResource::Sampler({
                images
                    .get(&self.0.atlas_texture)
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
