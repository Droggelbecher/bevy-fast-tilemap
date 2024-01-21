use crate::map::{Map, MapLoading};
use bevy::{prelude::*, sprite::Mesh2dHandle};

// Bundle of components you should typically have for a map.
// In addition to the components here you should either:
// a) insert a [`crate::map::MeshManagedByMap`] or
// b) override the [`bevy::sprite::Mesh2dHandle`] component with a custom mesh yourself (see
// `examples/custom_mesh.rs`)
/*
#[derive(Bundle, Clone, Default)]
pub struct MapBundle {
    material_mesh_bundle: MaterialMeshBundle,
    pub loading: MapLoading,

    //pub mesh: Mesh2dHandle,
    //pub map: Map,
    //pub transform: Transform,
    //pub global_transform: GlobalTransform,
    //pub visibility: Visibility,
    //pub inherited_visibility: InheritedVisibility,
}

impl MapBundle {
    pub fn new(map: Map) -> Self {
        Self {
            MaterialMeshBundle {
                material: materials.add(map)
            map, ..default() }
    }
}
*/
