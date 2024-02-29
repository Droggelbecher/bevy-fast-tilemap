use crate::map::{Map, MapAttributes, MapLoading, MeshManagedByMap};
use bevy::{prelude::*, sprite::Mesh2dHandle};

// Bundle of components you should typically have for a map.
#[derive(Bundle, Clone, Default)]
pub struct MapBundleUnmanaged {
    pub loading: MapLoading,
    pub attributes: MapAttributes,

    pub material: Handle<Map>,
    pub mesh: Mesh2dHandle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl MapBundleUnmanaged {
    pub fn new(map: Map, materials: &mut Assets<Map>) -> Self {
        Self {
            material: materials.add(map),
            ..default()
        }
    }
}

// Bundle of components you should typically have for a map.
#[derive(Bundle, Clone, Default)]
pub struct MapBundleManaged {
    pub loading: MapLoading,
    pub attributes: MapAttributes,

    pub material: Handle<Map>,
    pub mesh: Mesh2dHandle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,

    pub mesh_managed_by_map: MeshManagedByMap,
}

impl MapBundleManaged {
    pub fn new(map: Map, materials: &mut Assets<Map>) -> Self {
        Self {
            material: materials.add(map),
            ..default()
        }
    }
}
