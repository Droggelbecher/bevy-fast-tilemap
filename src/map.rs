use bevy::{
    prelude::*,
    render::{
        render_resource::{FilterMode, SamplerDescriptor},
        texture::ImageSampler,
    },
    sprite::Mesh2dHandle,
};
use std::num::NonZeroU32;

use crate::map_uniform::MapUniform;

/// Map, of size `size` tiles.
/// The actual tile data is stored in MapLayer components and Images in the asset system
/// and connected to this Map via bevys parent/child relationships for entities.
#[derive(Debug, Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct Map {
    /// Stores all the data that goes into the shader uniform,
    /// such as projection data, offsets, sizes, etc..
    pub(crate) map_uniform: MapUniform,

    /// Texture containing the tile IDs (one per each pixel)
    pub map_texture: Handle<Image>,

    /// Atlas texture with the individual tiles
    pub atlas_texture: Handle<Image>,
    // True iff the necessary images for this map are loaded
}

/// For entities that have all of:
/// - This component
/// - `Map`
/// - `Mesh2dHandle`
/// The Mesh will be automatically replaced with a rectangular mesh matching
/// the bounding box of the `Map` whenever a map change is detected.
#[derive(Debug, Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct MeshManagedByMap;

/// `Map` Entities that also have this component are in need of recomputing
/// some internal state (using `map.update()`).
#[derive(Debug, Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct MapDirty;

impl Map {
    /// Dimensions of this map in tiles.
    pub fn map_size(&self) -> UVec2 {
        self.map_uniform.map_size()
    }

    /// Size of the map contents bounding box in world coordinates
    pub fn world_size(&self) -> Vec2 {
        self.map_uniform.world_size()
    }

    /// Convert map position in `[(0.0, 0.0) .. self.size)`
    /// to world position.
    /// E.g. map position `(0.5, 0.5)` is in the center of the tile
    /// at index `(0, 0)`.
    pub fn map_to_world(&self, map_position: Vec2) -> Vec2 {
        self.map_uniform.map_to_world(map_position)
    }

    /// Convert world position to map position.
    pub fn world_to_map(&self, world: Vec2) -> Vec2 {
        self.map_uniform.world_to_map(world)
    }

    /// Use this to avoid creating a mutable reference 
    pub fn needs_update(&self, images: &Assets<Image>) -> bool {
        let map_texture = match images.get(&self.map_texture) {
            Some(x) => x,
            None => {
                // No point in updating anything yet, lets wait until the texture is there
                return false;
            }
        };

        let atlas_texture = match images.get(&self.atlas_texture) {
            Some(x) => x,
            None => {
                // No point in updating anything yet, lets wait until the texture is there
                return false;
            }
        };

        self.map_uniform
            .needs_update(map_texture.size().as_uvec2(), atlas_texture.size())
    }

    pub fn set_dirty(&mut self) {
        self.map_uniform.set_dirty();
    }

    /// Update internal state.
    /// Should not be necessary to call this if only map contents changed.
    pub fn update(&mut self, images: &Assets<Image>) -> bool {
        let map_texture = match images.get(&self.map_texture) {
            Some(x) => x,
            None => {
                warn!("No map texture");
                self.map_uniform.set_dirty();
                return false;
            }
        };

        let atlas_texture = match images.get(&self.atlas_texture) {
            Some(x) => x,
            None => {
                warn!("No atlas texture");
                self.map_uniform.set_dirty();
                return false;
            }
        };

        self.map_uniform
            .update(map_texture.size().as_uvec2(), atlas_texture.size())
    }

    /// Get mutable access to map layers via a `MapIndexer`.
    /// For this needs to mutably borrow the contained `MapLayer`s
    /// and the associated `Image`s.
    ///
    /// ```
    /// fn some_system(
    ///    mut images: ResMut<Assets<Image>>,
    ///    mut maps: Query<(&mut Map, &Transform)>,
    ///    // ...
    /// ) {
    ///
    ///   // Obtain mutable access to the underlying image structure.
    ///   // Use this only when you intend to make modifications to avoid
    ///   // unnecessary data transferns to the GPU.
    ///   if let Ok(m) = map.get_mut(&mut *images) {
    ///     // Set tile at (x, y) to tileset index 3
    ///     m[ivec2(x, y)] = 3;
    ///   }
    /// }
    /// ```
    pub fn get_mut<'a>(
        &self,
        images: &'a mut Assets<Image>,
    ) -> Result<MapIndexer<'a>, &'a str> {
        let image = images
            .get_mut(&self.map_texture)
            .ok_or("Map texture not yet loaded")?;

        Ok(MapIndexer {
            image,
            size: self.map_uniform.map_size(),
        })
    } // get_mut()
} // impl Map

/// Indexer into a map.
/// Internally holds a mutable reference to the underlying texture.
/// See `Map.get_mut` for a usage example.
#[derive(Debug)]
pub struct MapIndexer<'a> {
    pub(crate) image: &'a mut Image,
    pub(crate) size: UVec2,
}

impl<'a> MapIndexer<'a> {
    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn at_ivec(&self, i: IVec2) -> u16 {
        self.at(i.x as u32, i.y as u32)
    }

    pub fn at_uvec(&self, i: UVec2) -> u16 {
        self.at(i.x, i.y)
    }

    pub fn at(&self, x: u32, y: u32) -> u16 {
        let idx = y as isize * self.size.x as isize + x as isize;
        unsafe {
            let ptr = self.image.data.as_ptr() as *const u16;
            *ptr.offset(idx)
        }
    }

    pub fn set_uvec(&mut self, i: UVec2, v: u16) {
        self.set(i.x, i.y, v)
    }

    pub fn set(&mut self, x: u32, y: u32, v: u16) {
        let idx = y as isize * self.size.x as isize + x as isize;
        unsafe {
            let ptr = self.image.data.as_ptr() as *mut u16;
            *ptr.offset(idx) = v
        }
    }
}

/// Signals that `self.map` has been fully loaded (materials & images),
/// so all layer's .get and .set methods can be used.
#[derive(Debug)]
pub struct MapReadyEvent {
    pub map: Entity,
}

/// Check maps/textures state and mark them with `MapDirty` if they need an update
pub fn mark_maps_dirty(
    maps: Query<(Ref<Map>, Entity, Option<&MeshManagedByMap>), Without<MapDirty>>,
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    for ev in ev_asset.iter() {
        for (map, map_entity, _) in maps.iter() {
            match ev {
                AssetEvent::Created { handle }
                    if *handle == map.map_texture || *handle == map.atlas_texture =>
                {
                    // Set some sampling options for the atlas texture for nicer looks,
                    // such as avoiding "grid lines" when zooming out or mushy edges.
                    //
                    if let Some(atlas) = images.get_mut(&map.atlas_texture) {
                        // the below seems to crash?
                        //atlas.texture_descriptor.mip_level_count = 3;
                        atlas.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                            // min_filter of linear gives undesired grid lines when zooming out
                            min_filter: FilterMode::Nearest,
                            // mag_filter of linear gives mushy edges on tiles in closeup which is
                            // usually not what we want
                            mag_filter: FilterMode::Nearest,
                            mipmap_filter: FilterMode::Linear,
                            ..default()
                        });

                        if let Some(ref mut view_descriptor) = atlas.texture_view_descriptor {
                            view_descriptor.mip_level_count = NonZeroU32::new(4u32);
                        }
                    }

                    commands.entity(map_entity).insert(MapDirty);
                }
                AssetEvent::Modified { handle } | AssetEvent::Removed { handle }
                    if *handle == map.map_texture || *handle == map.atlas_texture =>
                {
                    commands.entity(map_entity).insert(MapDirty);
                }
                _ => (),
            } // match ev
        } // for map
    } // for ev

    for (map, entity, _manage_mesh) in maps.iter() {
        if map.is_changed() || map.is_added() {
            commands.entity(entity).insert(MapDirty);
        }
    }
} // update_maps()

pub fn update_dirty_maps(
    images: Res<Assets<Image>>,
    mut maps: Query<(&mut Map, Entity, Option<&MeshManagedByMap>), With<MapDirty>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut send_map_ready_event: EventWriter<MapReadyEvent>,
) {
    for (mut map, entity, manage_mesh) in maps.iter_mut() {
        commands.entity(entity).remove::<MapDirty>();

        if !map.needs_update(images.as_ref()) {
            continue;
        }

        if map.update(images.as_ref()) {
            if let Some(_) = manage_mesh {
                let mesh = Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
                    size: map.world_size(),
                    flip: false,
                })));
                commands.entity(entity).insert(mesh);
            }

            send_map_ready_event.send(MapReadyEvent { map: entity });
        }
    }
}



