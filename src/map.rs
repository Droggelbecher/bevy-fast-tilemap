use bevy::sprite::Material2d;
use bevy::{
    math::{vec2, Vec3Swizzles},
    prelude::*,
    render::render_resource::ShaderRef,
    render::{
        render_resource::{AsBindGroup, SamplerDescriptor},
        texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    },
    sprite::Mesh2dHandle,
};

use crate::{map_builder::MapBuilder, map_uniform::MapUniform};

/// Map, holding handles to a map texture with the tile data and an atlas texture
/// with the tile renderings.
#[derive(Asset, Debug, Component, Clone, Default, Reflect, AsBindGroup)]
#[reflect(Component)]
pub struct Map {
    /// Stores all the data that goes into the shader uniform,
    /// such as projection data, offsets, sizes, etc..
    #[uniform(0)]
    pub(crate) map_uniform: MapUniform,

    /// Texture containing the tile IDs (one per each pixel)
    #[storage(100)]
    //pub map_texture: Handle<Image>,
    pub(crate) map_texture: Vec<u32>,

    /// Atlas texture with the individual tiles
    #[texture(101)]
    #[sampler(102)]
    pub(crate) atlas_texture: Handle<Image>,
    // True iff the necessary images for this map are loaded
}

impl Material2d for Map {
    fn fragment_shader() -> ShaderRef {
        "tilemap_shader.wgsl".into()
    }
}

/// For entities that have all of:
/// - This component
/// - [`Map`]
/// - [`bevy::sprite::Mesh2dHandle`]
/// The Mesh will be automatically replaced with a rectangular mesh matching
/// the bounding box of the `Map` whenever a map change is detected.
///
/// This is convenient if you dont care about the exact dimensions / shape of your mesh
/// but just want to be sure it holds the full map.
#[derive(Debug, Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct MeshManagedByMap;

/// Component temporarily active during map loading.
/// Will be added by [`crate::bundle::MapBundle`] and will automatically be removed, once the map is loaded.
#[derive(Debug, Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct MapLoading;

impl Map {
    /// Create a [`MapBuilder`] for configuring your map.
    pub fn builder(map_size: UVec2, atlas_texture: Handle<Image>, tile_size: Vec2) -> MapBuilder {
        MapBuilder::new(map_size, atlas_texture, tile_size)
    }

    /// Dimensions of this map in tiles.
    pub fn map_size(&self) -> UVec2 {
        self.map_uniform.map_size()
    }

    /// Size of the map contents bounding box in world coordinates
    pub fn world_size(&self) -> Vec2 {
        self.map_uniform.world_size()
    }

    /// Convert map position in `[(0.0, 0.0) .. self.size)`
    /// to local world position (before this entities transform).
    /// E.g. map position `(0.5, 0.5)` is in the center of the tile
    /// at index `(0, 0)`.
    pub fn map_to_local(&self, map_position: Vec2) -> Vec2 {
        self.map_uniform.map_to_local(map_position.extend(0.0)).xy()
    }

    /// Same as [`Self::map_to_local`], but return a 3d coordinate,
    /// z-value is the logical "depth" of the map position (for eg axonometric projection).
    /// Not generally consistent with actual z-position of the mesh.
    pub fn map_to_local_3d(&self, map_position: Vec3) -> Vec3 {
        self.map_uniform.map_to_local(map_position)
    }

    pub fn map_to_world_3d(&self, map_position: Vec3) -> Vec3 {
        self.map_uniform.map_to_world(map_position)
    }

    /// Convert world position to map position.
    pub fn world_to_map(&self, world: Vec2) -> Vec2 {
        self.map_uniform.world_to_map(world.extend(0.0)).xy()
    }

    pub fn world_to_map_3d(&self, world: Vec3) -> Vec3 {
        self.map_uniform.world_to_map(world)
    }

    /// Use this to avoid creating a mutable reference
    pub fn needs_update(&self, images: &Assets<Image>) -> bool {
        /*
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

        self.map_uniform.map_size != map_texture.size() //.as_uvec2()
            || self.map_uniform.atlas_size != atlas_texture.size().as_vec2()
        */

        // TODO

        true
    }

    pub fn is_loaded(&self, images: &Assets<Image>) -> bool {
        /*
        if images.get(&self.map_texture).is_none() {
            return false;
        }
        */

        if images.get(&self.atlas_texture).is_none() {
            return false;
        }
        // TODO

        true
    }

    /// Update internal state.
    /// Call this when map size changed or assets may have become available.
    /// Should not be necessary to call this if only map contents changed.
    pub fn update(&mut self, images: &Assets<Image>) -> bool {
        /*
        let map_texture = match images.get(&self.map_texture) {
            Some(x) => x,
            None => {
                warn!("No map texture");
                return false;
            }
        };

        let a = self.map_uniform.update_map_size(map_texture.size());
        let b = self
            .map_uniform
            .update_atlas_size(atlas_texture.size().as_vec2());

        a || b
        */
        // TODO
        let atlas_texture = match images.get(&self.atlas_texture) {
            Some(x) => x,
            None => {
                warn!("No atlas texture");
                return false;
            }
        };

        let b = self
            .map_uniform
            .update_atlas_size(atlas_texture.size().as_vec2());
        b
    }

    // Get mutable access to map layers via a `MapIndexer`.
    // For this needs to mutably borrow the referenced
    // `Image`s.
    //
    // ```
    // fn some_system(
    //    mut images: ResMut<Assets<Image>>,
    //    mut maps: Query<(&mut Map, &Transform)>,
    //    // ...
    // ) {
    //
    //   // Obtain mutable access to the underlying image structure.
    //   // Use this only when you intend to make modifications to avoid
    //   // unnecessary data transferns to the GPU.
    //   if let Ok(m) = map.get_mut(&mut *images) {
    //     // Set tile at (x, y) to tileset index 3
    //     m.set(x, y, 3);
    //   }
    // }
    // ```
    // TODO
    //pub fn get_mut<'a>(&self, images: &'a mut Assets<Image>) -> Result<MapIndexer<'a>, &'a str> {
    //let image = images
    //.get_mut(&self.map_texture)
    //.ok_or("Map texture not yet loaded")?;

    //Ok(MapIndexer {
    //image,
    //size: self.map_uniform.map_size(),
    //})
    //} // get_mut()
} // impl Map

// Indexer into a map.
// Internally holds a mutable reference to the underlying texture.
// See [`Map::get_mut`] for a usage example.
#[derive(Debug)]
pub struct MapIndexer<'a> {
    pub(crate) map: &'a mut Map,
    //pub(crate) image: &'a mut Image,
    //// TODO: We can get size from image.texture_descriptor.size
    //pub(crate) size: UVec2,
}

impl<'a> MapIndexer<'a> {
    /// Size of the map being indexed.
    pub fn size(&self) -> UVec2 {
        self.map.map_size()
    }

    /// Get tile at given position.
    pub fn at_ivec(&self, i: IVec2) -> u32 {
        self.at(i.x as u32, i.y as u32)
    }

    ///// Get tile at given position.
    pub fn at_uvec(&self, i: UVec2) -> u32 {
        self.at(i.x, i.y)
    }

    /// Get tile at given position.
    pub fn at(&self, x: u32, y: u32) -> u32 {
        self.assert_size();
        let idx = y as usize * self.size().x as usize + x as usize;
        self.map.map_texture[idx]
    }

    /// Set tile at given position.
    pub fn set_uvec(&mut self, i: UVec2, v: u32) {
        self.set(i.x, i.y, v)
    }

    /// Set tile at given position.
    pub fn set(&mut self, x: u32, y: u32, v: u32) {
        self.assert_size();
        let idx = y as usize * self.size().x as usize + x as usize;
        self.map.map_texture[idx] = v;
    }

    fn assert_size(&self) {
        let s = (self.map.map_size().x * self.map.map_size().y) as usize;
        assert!(self.map.map_texture.len() == s);
    }
}

/// Signals that the given map has been fully loaded and from now on
/// [`Map::get_mut`] should be successful.
#[derive(Debug, Event)]
pub struct MapReadyEvent {
    pub map: Entity,
}

///
pub fn configure_loaded_assets(
    //maps: Query<Ref<Map>>,
    map_materials: ResMut<Assets<Map>>,
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    // TODO: Configure sampler etc.. for atlas image with the new asset preprocessing system?
    /*
    for ev in ev_asset.iter() {
        for map in maps.iter() {
            match ev {
                AssetEvent::Added { id }
                    if *id == map.atlas_texture.id() =>
                {

                    // Set some sampling options for the atlas texture for nicer looks,
                    // such as avoiding "grid lines" when zooming out or mushy edges.
                    //
                    if let Some(atlas) = images.get_mut(&map.atlas_texture) {
                        // the below seems to crash?
                        //atlas.texture_descriptor.mip_level_count = 3;
                        atlas.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            // min_filter of linear gives undesired grid lines when zooming out
                            min_filter: ImageFilterMode::Nearest,
                            // mag_filter of linear gives mushy edges on tiles in closeup which is
                            // usually not what we want
                            mag_filter: ImageFilterMode::Nearest,
                            mipmap_filter: ImageFilterMode::Linear,
                            ..default()
                        });

                        if let Some(ref mut view_descriptor) = atlas.texture_view_descriptor {
                            view_descriptor.mip_level_count = Some(4);
                        }
                    }
                }
                _ => (),
            } // match ev
        } // for map
    } // for ev
    */
} // configure_loaded_assets()

/// Check to see if any maps' assets became available and send a MapReadyEvent
/// if so.
pub fn update_loading_maps(
    images: Res<Assets<Image>>,
    mut map_materials: ResMut<Assets<Map>>,
    // TODO: We should have a general marker component that
    // could also duplicate some of the materials metadata if need be
    // (at the cost of risking to be out of sync with the material)
    mut maps: Query<(Entity, &Handle<Map>, Option<&MeshManagedByMap>), With<MapLoading>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    //mut send_map_ready_event: EventWriter<MapReadyEvent>,
) {
    // TODO
    for (entity, map_handle, manage_mesh) in maps.iter_mut() {
        let map = match map_materials.get_mut(map_handle) {
            Some(x) => x,
            None => {
                warn!("No map material");
                continue;
            }
        };

        //if map.is_loaded(images.as_ref()) {
        commands.entity(entity).remove::<MapLoading>();
        map.update(images.as_ref());

        if manage_mesh.is_some() {
            let mesh = Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
                // TODO DEBUG
                size: map.world_size(),
                flip: false,
            })));
            commands.entity(entity).insert(mesh);
        }

        debug!("Map loaded: {:?}", map.map_size());
        //send_map_ready_event.send(MapReadyEvent { map: entity });
        //}
    }
}

pub fn apply_map_transforms(//mut maps: Query<(&mut Map, &GlobalTransform), Changed<GlobalTransform>>,
) {
    // TODO
    //for (mut map, transform) in &mut maps {
    //map.map_uniform.apply_transform(transform.clone());
    //}
}
