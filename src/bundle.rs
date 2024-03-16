use crate::map::{Map, MapAttributes, MapLoading, MeshManagedByMap};
use bevy::{
    prelude::*,
    render::render_resource::{encase::internal::WriteInto, AsBindGroup, ShaderSize, ShaderType},
    sprite::Mesh2dHandle,
};

// Bundle of components you should typically have for a map.
#[derive(Bundle, Clone, Default)]
pub struct MapBundleUnmanaged<UserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    pub loading: MapLoading,
    pub attributes: MapAttributes,

    pub material: Handle<Map<UserData>>,
    pub mesh: Mesh2dHandle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl<UserData> MapBundleUnmanaged<UserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    pub fn new(map: Map<UserData>, materials: &mut Assets<Map<UserData>>) -> Self {
        Self {
            material: materials.add(map),
            ..default()
        }
    }
}

// Bundle of components you should typically have for a map.
#[derive(Bundle, Clone, Default)]
pub struct MapBundleManaged<UserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    pub loading: MapLoading,
    pub attributes: MapAttributes,

    pub material: Handle<Map<UserData>>,
    pub mesh: Mesh2dHandle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,

    pub mesh_managed_by_map: MeshManagedByMap,
}

impl<UserData> MapBundleManaged<UserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    pub fn new(map: Map<UserData>, materials: &mut Assets<Map<UserData>>) -> Self {
        Self {
            material: materials.add(map),
            ..default()
        }
    }
}
