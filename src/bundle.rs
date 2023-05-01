use crate::map::{Map, MapIndexer, MapDirty};
use crate::map_uniform::MapUniform;
use bevy::{
    math::{mat2, uvec2, vec2},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    sprite::Mesh2dHandle,
};
use std::mem::size_of;

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

