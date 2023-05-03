use crate::map::{Map, MapDirty};
use bevy::{
    prelude::*,
    sprite::Mesh2dHandle,
};

#[derive(Bundle, Clone, Default)]
pub struct MapBundle {
    pub mesh: Mesh2dHandle,
    pub map: Map,
    pub dirty: MapDirty,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl MapBundle {
    pub fn new(map: Map) -> Self {
        Self {
            map,
            ..default()
        }
    }
}

