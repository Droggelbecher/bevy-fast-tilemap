use crate::map::{Map, MapLoading, MeshManagedByMap};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::{prelude::*, sprite::Mesh2dHandle};

// Bundle of components you should typically have for a map.
// In addition to the components here you should either:
// a) insert a [`crate::map::MeshManagedByMap`] or
// b) override the [`bevy::sprite::Mesh2dHandle`] component with a custom mesh yourself (see
// `examples/custom_mesh.rs`)
#[derive(Bundle, Clone, Default)]
pub struct MapBundle {
    //pub material_mesh2d_bundle: MaterialMesh2dBundle<Map>,
    pub loading: MapLoading,
    pub mesh_managed_by_map: MeshManagedByMap,

    pub material: Handle<Map>,
    pub mesh: Mesh2dHandle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl MapBundle {
    pub fn new(map: Map, materials: &mut Assets<Map>) -> Self {
        Self {
            material: materials.add(map),
            ..default()
        }
    }
}
