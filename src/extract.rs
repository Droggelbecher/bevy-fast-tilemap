
use bevy::{
    prelude::*,
    render::Extract
};

use crate::map::Map;

#[derive(Component)]
pub struct ExtractedMap {
    /// Size of the map, in tiles.
    pub size: IVec2,

    /// Size of each tile, in pixels.
    pub tile_size: Vec2,

    /// Texture containing the tile IDs (one per each pixel)
    pub map_texture: Handle<Image>, //R16Uint

    /// Atlas texture with the individual tiles
    pub tiles_texture: Handle<Image>,
}

impl From<&Map> for ExtractedMap {
    fn from(map: &Map) -> Self {
        Self {
            size: map.size,
            tile_size: map.tile_size,
            map_texture: map.map_texture.clone(),
            tiles_texture: map.tiles_texture.clone(),
        }
    }
}


/// Extract map data from the main world and copy it to the render world.
///
/// This is a system in the render app.
///
/// In our case this is simply cloning the `Map` entity.
///
pub fn extract_fast_tilemap(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    // When extracting, you must use `Extract` to mark the `SystemParam`s
    // which should be taken from the main world.
    query: Extract<Query<(Entity, &ComputedVisibility, &Map)>>,
) {
    // TODO: Can we avoid the reallocation here in the common case?
    let mut values = Vec::with_capacity(*previous_len);

    for (entity, computed_visibility, map) in &query {
        if !computed_visibility.is_visible() {
            continue;
        }
        values.push((entity, ExtractedMap::from(map)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}
