use bevy::{prelude::*, render::Extract};

use crate::map::Map;

#[derive(Debug, Component, Clone)]
pub struct ExtractedMap(pub Map);

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
        values.push((entity, ExtractedMap(map.clone())));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}
