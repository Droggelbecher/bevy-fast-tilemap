use bevy::{
    prelude::*,
    render::{
        render_resource::{SamplerDescriptor, FilterMode},
        texture::ImageSampler
    },
};
use std::ops::{Index, IndexMut};
use std::num::NonZeroU32;

/// Map, of size `size` tiles.
/// The actual tile data is stored in MapLayer components and Images in the asset system
/// and connected to this Map via bevys parent/child relationships for entities.
#[derive(Debug, Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct Map {
    /// Size of the map, in tiles.
    pub size: IVec2,

    /// Size of each tile, in pixels.
    pub tile_size: Vec2,

    /// Texture containing the tile IDs (one per each pixel)
    pub map_texture: Handle<Image>,

    /// Atlas texture with the individual tiles
    pub tiles_texture: Handle<Image>,

    /// fractional 2d map index -> world pos
    pub projection: Mat2,

    /// world pos -> fractional 2d map index
    pub inverse_projection: Mat2,

    /// Offset of the projected map in world coordinates
    pub world_offset: Vec2,

    /// relative anchor point position in a tile (in [0..1]^2)
    pub tile_anchor_point: Vec2,

    /// True iff the necessary images for this map are loaded
    pub ready: bool,
}

impl Map {
    /// Dimensions of this map in tiles.
    pub fn size(&self) -> IVec2 {
        self.size
    }

    /// Convert map position in `[(0.0, 0.0) .. self.size)`
    /// to world position.
    /// E.g. map position `(0.5, 0.5)` is in the center of the tile
    /// at index `(0, 0)`.
    pub fn map_to_world(&self, map_position: Vec2) -> Vec2 {
        (self.projection * map_position) * self.tile_size + self.world_offset
    }

    /// Convert world position to map position.
    pub fn world_to_map(&self, world: Vec2) -> Vec2 {
        self.inverse_projection * ((world - self.world_offset) / self.tile_size)
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
        &mut self,
        images: &'a mut Assets<Image>,
    ) -> Result<MapIndexer<'a>, &'a str> {
        let image = images
            .get_mut(&self.map_texture)
            .ok_or("Map texture not yet loaded")?;

        Ok(MapIndexer {
            image,
            size: self.size,
        })
    } // get_mut()
} // impl Map

/// Indexer into a map.
/// Internally holds a mutable reference to the underlying texture.
/// See `Map.get_mut` for a usage example.
#[derive(Debug)]
pub struct MapIndexer<'a> {
    pub(crate) image: &'a mut Image,
    pub(crate) size: IVec2,
}

impl<'a> MapIndexer<'a> {
    pub fn size(&self) -> IVec2 { self.size }

    /*
    pub fn resize(&mut self, size: IVec2) {
        // TODO: Also resize the mesh somehow?!
        // Consider removing this and have people instead create a new map
        // when they want to change size?
        //
        self.image.resize(bevy::render::render_resource::Extent3d {
            width: size.x as u32,
            height: size.y as u32,
            depth_or_array_layers: 1,
        });
        *self.size = size;
    }
    */
}

impl<'a> Index<IVec2> for MapIndexer<'a> {
    type Output = u16;

    /// Index the map by 2d integer coordinate.
    /// Expected to be in `[(0, 0) .. map_size)`
    fn index(&self, i: IVec2) -> &Self::Output {
        let idx = i.y as isize * self.size.x as isize + i.x as isize;
        // TODO: Get rid of this unsafe
        unsafe {
            let ptr = self.image.data.as_ptr() as *const u16;
            &*ptr.offset(idx)
        }
    }
}

impl<'a> IndexMut<IVec2> for MapIndexer<'a> {

    /// Index the map by 2d integer coordinate.
    /// Expected to be in `[(0, 0) .. map_size)`
    fn index_mut(&mut self, i: IVec2) -> &mut u16 {
        let idx = i.y as isize * self.size.x as isize + i.x as isize;
        // TODO: Get rid of this unsafe
        unsafe {
            let ptr = self.image.data.as_ptr() as *mut u16;
            &mut *ptr.offset(idx)
        }
    }
}

/// Signals that `self.map` has been fully loaded (materials & images),
/// so all layer's .get and .set methods can be used.
#[derive(Debug)]
pub struct MapReadyEvent {
    pub map: Entity,
}

/// Check if all images and materials of a `Map` are loaded,
/// when thats the case, send out a `MapReadyEvent`.
pub fn check_map_ready_events(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    mut send_map_ready_event: EventWriter<MapReadyEvent>,
    mut maps: Query<(&mut Map, Entity)>,
) {
    for ev in ev_asset.iter() {
        for (mut map, map_entity) in maps.iter_mut() {
            match ev {
                AssetEvent::Created { handle }
                    if *handle == map.map_texture || *handle == map.tiles_texture =>
                {
                    if let Some(tiles) = images.get_mut(&map.tiles_texture) {
                        // the below seems to crash?
                        //tiles.texture_descriptor.mip_level_count = 3;
                        tiles.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                            // min_filter of linear gives undesired grid lines when zooming out
                            min_filter: FilterMode::Nearest,
                            // mag_filter of linear gives mushy edges on tiles in closeup which is
                            // usually not what we want
                            mag_filter: FilterMode::Nearest,
                            mipmap_filter: FilterMode::Linear,
                            ..default()
                        });

                        if let Some(ref mut view_descriptor) = tiles.texture_view_descriptor {
                            view_descriptor.mip_level_count = NonZeroU32::new(4u32);
                        }
                    }

                    if images.get(&map.map_texture).is_some()
                        && images.get(&map.tiles_texture).is_some()
                    {
                        send_map_ready_event.send(MapReadyEvent { map: map_entity });
                        map.ready = true;
                    } // if
                } // AssetEvent::Created
                _ => {}
            } // match
        } // ev
    } // map
} // handle_tilemap_events()
