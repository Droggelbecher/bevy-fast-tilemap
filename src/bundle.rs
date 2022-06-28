use crate::map::{Map, MapLayer, MapLayerMaterial};
use bevy::{
    math::vec2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    sprite::Mesh2dHandle,
};
use std::mem::size_of;

/// Descriptor for creating a fast tilemap bundle
pub struct FastTileMapDescriptor {
    /// Size of the map (in tiles)
    pub map_size: IVec2,
    /// Size of a single tile (in pixels)
    pub tile_size: Vec2,
    /// Images holding the texture atlases, one for each layer of the map.
    /// All atlases must have a tile size of `tile_size` and no padding.
    pub tiles_textures: Vec<Handle<Image>>,
}

impl FastTileMapDescriptor {
    /// Create and spawn an entity for the described map.
    /// Create and spawn child entities for each layer.
    pub fn spawn(
        self,
        commands: &mut Commands,
        images: &mut ResMut<Assets<Image>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<MapLayerMaterial>>,
    ) -> Entity {
        let mesh = Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
            size: vec2(
                self.map_size.x as f32 * self.tile_size.x,
                self.map_size.y as f32 * self.tile_size.y,
            ),
            flip: false,
        })));

        let mut bundle = FastTileMapBundle {
            mesh: mesh.clone(),
            map: Map {
                size: self.map_size,
                tile_size: self.tile_size,
                ..default()
            },
            ..default()
        };

        for tiles_texture in self.tiles_textures.iter() {
            let mut map_image = Image::new(
                Extent3d {
                    width: self.map_size.x as u32,
                    height: self.map_size.y as u32,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![0u8; (self.map_size.x * self.map_size.y) as usize * size_of::<u16>()],
                TextureFormat::R16Uint,
            );
            map_image.texture_descriptor.usage =
                TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST;
            map_image.texture_descriptor.mip_level_count = 1;

            let mut tint_image = Image::new(
                Extent3d {
                    width: self.map_size.x as u32,
                    height: self.map_size.y as u32,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![255u8; (self.map_size.x * self.map_size.y) as usize * 4 * size_of::<u8>()],
                TextureFormat::Rgba8Uint,
            );
            tint_image.texture_descriptor.usage =
                TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST;
            tint_image.texture_descriptor.mip_level_count = 1;

            let layer = MapLayer {
                material: materials
                    .add(MapLayerMaterial {
                        map_texture: images.add(map_image),
                        tint_texture: images.add(tint_image),
                        tiles_texture: tiles_texture.clone(),
                        tile_size: self.tile_size,
                        tile_ids: 0,
                        ready: false,
                    })
                    .into(),
            };
            let layer_entity = commands.spawn_bundle((
                layer.material.clone(),
                layer,
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                ComputedVisibility::default(),
                mesh.clone(),
            )).id();

            bundle.map.layers.push(layer_entity);

        }

        commands
            .spawn_bundle(bundle.clone())
            .push_children(&bundle.map.layers[..])
            .id()
    } // fn spawn()
} // impl FastTileMapDescriptor

#[derive(Bundle, Clone)]
pub struct FastTileMapBundle {
    pub mesh: Mesh2dHandle,
    pub map: Map,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Default for FastTileMapBundle {
    fn default() -> Self {
        Self {
            mesh: default(),
            map: default(),
            transform: default(),
            global_transform: default(),
            visibility: default(),
            computed_visibility: default(),
        }
    }
}
