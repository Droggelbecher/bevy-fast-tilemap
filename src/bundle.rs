use crate::map::Map;
use bevy::{
    ecs::system::EntityCommands,
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
    pub tiles_texture: Handle<Image>,
    pub transform: Transform,
}

impl FastTileMapDescriptor {

    pub fn spawn<'a, 'w, 's>(
        self,
        commands: &'a mut Commands<'w, 's>,
        images: &mut ResMut<Assets<Image>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> EntityCommands<'w, 's, 'a> {

        // See bevy_render/src/mesh/shape/mod.rs
        // will generate 3d position, 3d normal, and 2d UVs
        let mesh = Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
            size: vec2(
                self.map_size.x as f32 * self.tile_size.x,
                self.map_size.y as f32 * self.tile_size.y,
            ),
            flip: false,
        })));

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
            TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST
            | TextureUsages::TEXTURE_BINDING;
        map_image.texture_descriptor.mip_level_count = 1;

        let bundle = FastTileMapBundle {
            mesh: mesh.clone(),
            map: Map {
                size: self.map_size,
                tile_size: self.tile_size,
                map_texture: images.add(map_image),
                tiles_texture: self.tiles_texture.clone(),
                ready: false,
            },
            transform: self.transform,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        };

        commands.spawn(bundle)
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
