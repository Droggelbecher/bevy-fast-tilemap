use bevy::{
    math::{dmat2, vec2, Vec3Swizzles},
    prelude::*,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{
            encase::internal::WriteInto, AsBindGroup, ShaderDefVal, ShaderRef, ShaderSize,
            ShaderType, VertexFormat,
        },
        texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    },
    sprite::{Material2d, Mesh2dHandle},
};

use crate::{map_builder::MapBuilder, map_uniform::MapUniform, shader::SHADER_HANDLE};

const ATTRIBUTE_MAP_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("MapPosition", 988779054, VertexFormat::Float32x2);
const ATTRIBUTE_MIX_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("MixColor", 988779055, VertexFormat::Float32x4);
const ATTRIBUTE_ANIMATION_STATE: MeshVertexAttribute =
    MeshVertexAttribute::new("AnimationState", 988779056, VertexFormat::Float32);

#[derive(Debug, Clone, Default, Reflect, AsBindGroup, ShaderType)]
pub struct DefaultUserData {
    x: u32,
}

/// Map, holding handles to a map texture with the tile data and an atlas texture
/// with the tile renderings.
#[derive(Asset, Debug, Clone, Default, Reflect, AsBindGroup)]
#[bind_group_data(MapKey)]
pub struct Map<UserData = DefaultUserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    /// Stores all the data that goes into the shader uniform,
    /// such as projection data, offsets, sizes, etc..
    #[uniform(0)]
    pub(crate) map_uniform: MapUniform,

    #[uniform(1)]
    pub user_data: UserData,

    /// Texture containing the tile IDs (one per each pixel)
    #[storage(100, read_only)]
    pub(crate) map_texture: Vec<u32>,

    /// Atlas texture with the individual tiles
    #[texture(101)]
    #[sampler(102)]
    pub(crate) atlas_texture: Handle<Image>,

    /// Atlas texture with the individual tiles
    #[texture(103)]
    #[sampler(104)]
    pub(crate) pattern_atlas_texture: Handle<Image>,

    pub(crate) perspective_defs: Vec<String>,
    pub(crate) perspective_underhangs: bool,
    pub(crate) perspective_overhangs: bool,
    pub(crate) dominance_overhangs: bool,
    pub(crate) force_underhangs: Vec<Vec2>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct MapKey {
    pub(crate) perspective_defs: Vec<String>,
    pub(crate) perspective_underhangs: bool,
    pub(crate) perspective_overhangs: bool,
    pub(crate) dominance_overhangs: bool,
}

impl<UserData> From<&Map<UserData>> for MapKey
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    fn from(map: &Map<UserData>) -> Self {
        MapKey {
            perspective_defs: map.perspective_defs.clone(),
            perspective_underhangs: map.perspective_underhangs,
            perspective_overhangs: map.perspective_overhangs,
            dominance_overhangs: map.dominance_overhangs,
        }
    }
}

/// Per-vertex attributes for map
#[derive(Component, Default, Clone, Debug)]
pub struct MapAttributes {
    pub mix_color: Vec<Vec4>,
}

impl MapAttributes {
    fn set_mix_color(attributes: Option<&MapAttributes>, mesh: &mut Mesh) {
        let l = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().len();

        let mut v = vec![Vec4::ONE; l];
        if let Some(attr) = attributes {
            if attr.mix_color.len() > v.len() {
                v.resize(attr.mix_color.len(), Vec4::ONE);
            }
            for (i, c) in attr.mix_color.iter().enumerate() {
                v[i] = *c;
            }
        }

        mesh.insert_attribute(ATTRIBUTE_MIX_COLOR, v);
    }

    fn set_map_position<UserData>(
        _attributes: Option<&MapAttributes>,
        mesh: &mut Mesh,
        map: &Map<UserData>,
    ) where
        UserData: AsBindGroup
            + Reflect
            + Clone
            + Default
            + TypePath
            + ShaderType
            + WriteInto
            + ShaderSize,
    {
        let v: Vec<_> = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .iter()
            .map(|p| map.world_to_map(Vec2::new(p[0], p[1])))
            .collect();
        mesh.insert_attribute(ATTRIBUTE_MAP_POSITION, v);
    }

    fn set_animation_state(_attributes: Option<&MapAttributes>, mesh: &mut Mesh, time: &Time) {
        let l = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().len();
        let v = vec![time.elapsed_seconds_wrapped(); l];
        mesh.insert_attribute(ATTRIBUTE_ANIMATION_STATE, v);
    }
}

impl<UserData> Material2d for Map<UserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER_HANDLE)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER_HANDLE)
    }

    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_MAP_POSITION.at_shader_location(1),
            ATTRIBUTE_MIX_COLOR.at_shader_location(2),
            ATTRIBUTE_ANIMATION_STATE.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];

        let fragment = descriptor.fragment.as_mut().unwrap();

        if key.bind_group_data.perspective_underhangs {
            fragment.shader_defs.push(ShaderDefVal::Bool(
                "PERSPECTIVE_UNDERHANGS".to_string(),
                true,
            ));
        }

        if key.bind_group_data.perspective_overhangs {
            fragment.shader_defs.push(ShaderDefVal::Bool(
                "PERSPECTIVE_OVERHANGS".to_string(),
                true,
            ));
        }

        if key.bind_group_data.dominance_overhangs {
            fragment
                .shader_defs
                .push(ShaderDefVal::Bool("DOMINANCE_OVERHANGS".to_string(), true));
        }

        for def in key.bind_group_data.perspective_defs.iter() {
            fragment
                .shader_defs
                .push(ShaderDefVal::Bool(def.clone(), true));
        }

        debug!("{:?}", fragment.shader_defs);

        Ok(())
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

impl<UserData> Map<UserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    /// Create a [`MapBuilder`] for configuring your map.
    pub fn builder(
        map_size: UVec2,
        atlas_texture: Handle<Image>,
        tile_size: Vec2,
    ) -> MapBuilder<UserData> {
        MapBuilder::new(map_size, atlas_texture, tile_size)
    }

    pub fn indexer_mut(&mut self) -> MapIndexer<UserData> {
        MapIndexer::<UserData> { map: self }
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

    pub fn is_loaded(&self, images: &Assets<Image>) -> bool {
        images.get(&self.atlas_texture).is_some()
    }

    /// Update internal state.
    /// Call this when map size changed or assets may have become available.
    /// Should not be necessary to call this if only map contents changed.
    pub fn update(&mut self, images: &Assets<Image>) -> bool {
        let Some(atlas_texture) = images.get(&self.atlas_texture) else {
            return false;
        };

        let maybe_pattern_atlas_texture = images.get(&self.pattern_atlas_texture);

        if self.map_uniform.n_pattern_indices > 0 && maybe_pattern_atlas_texture.is_none() {
            return false;
        }

        self.map_uniform
            .update_atlas_size(
                atlas_texture.size().as_vec2(),
                maybe_pattern_atlas_texture.map(|t| t.size().as_vec2())
            )
    }

    pub(crate) fn update_inverse_projection(&mut self) {
        let projection2d = dmat2(
            self.map_uniform.projection.x_axis.xy().as_dvec2(),
            self.map_uniform.projection.y_axis.xy().as_dvec2(),
        );

        self.map_uniform.inverse_projection = projection2d.inverse().as_mat2();

        // Iterate through the four "straight" neighboring map directions, and figure
        // out which of these have negative Z-values after projection to the world.
        // These are exactly the directions we should "overlap" in the shader in perspective
        // overhang mode.
        let offsets = [
            (vec2(0.0, -1.0), "ZN"),
            (vec2(-1.0, -1.0), "NN"),
            (vec2(-1.0, 0.0), "NZ"),
            (vec2(-1.0, 1.0), "NP"),
            (vec2(0.0, 1.0), "ZP"),
            (vec2(1.0, 1.0), "PP"),
            (vec2(1.0, 0.0), "PZ"),
            (vec2(1.0, -1.0), "PN"),
        ];

        let mut defs = Vec::new();

        if self.force_underhangs.is_empty() {
            // Derive underhangs from perspective (z < 0) values
            for (offset, def) in offsets.iter() {
                if self.map_uniform.map_to_local(offset.extend(0.0)).z < 0.0 {
                    defs.push(format!("PERSPECTIVE_UNDER_{}", def));
                }
            }
        } else {
            // Use forced underhangs
            for direction in self.force_underhangs.iter() {
                for (offset, def) in offsets.iter() {
                    if direction.angle_between(*offset) == 0.0 {
                        defs.push(format!("PERSPECTIVE_UNDER_{}", def));
                    }
                }
            }
        }
        self.perspective_defs = defs;
    }
} // impl Map

// Indexer into a map.
// Internally holds a mutable reference to the underlying texture.
// See [`Map::get_mut`] for a usage example.
#[derive(Debug)]
pub struct MapIndexer<'a, UserData = DefaultUserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    pub(crate) map: &'a mut Map<UserData>,
}

impl<'a, UserData> MapIndexer<'a, UserData>
where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    /// Size of the map being indexed.
    pub fn size(&self) -> UVec2 {
        self.map.map_size()
    }

    /// Get tile at given position.
    pub fn at_ivec(&self, i: IVec2) -> u32 {
        self.at(i.x as u32, i.y as u32)
    }

    /// Get tile at given position.
    pub fn at_uvec(&self, i: UVec2) -> u32 {
        self.at(i.x, i.y)
    }

    /// Get tile at given position.
    pub fn at(&self, x: u32, y: u32) -> u32 {
        // ensure x/y do not go out of bounds individually
        if x >= self.size().x || y >= self.size().y {
            return 0;
        }
        let idx = y as usize * self.size().x as usize + x as usize;
        self.map.map_texture[idx]
    }

    /// Set tile at given position.
    pub fn set_uvec(&mut self, i: UVec2, v: u32) {
        self.set(i.x, i.y, v)
    }

    /// Set tile at given position.
    pub fn set(&mut self, x: u32, y: u32, v: u32) {
        // ensure x/y do not go out of bounds individually (even if the final index is in-bounds)
        if x >= self.size().x || y >= self.size().y {
            return;
        }
        let idx = y as usize * self.size().x as usize + x as usize;
        self.map.map_texture[idx] = v;
    }

    pub fn world_to_map(&self, world: Vec2) -> Vec2 {
        self.map.world_to_map(world)
    }

    pub fn map_to_world_3d(&self, map_position: Vec3) -> Vec3 {
        self.map.map_to_world_3d(map_position)
    }

    pub fn map_to_local_3d(&self, map_position: Vec3) -> Vec3 {
        self.map.map_to_local_3d(map_position)
    }

    pub fn map_to_local(&self, map_position: Vec2) -> Vec2 {
        self.map.map_to_local(map_position)
    }

    pub fn world_to_map_3d(&self, world: Vec3) -> Vec3 {
        self.map.world_to_map_3d(world)
    }
}

pub fn log_map_events<UserData>(
    mut ev_asset: EventReader<AssetEvent<Map<UserData>>>,
    map_handles: Query<&Handle<Map<UserData>>>,
) where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    for ev in ev_asset.read() {
        for map_handle in map_handles.iter() {
            match ev {
                AssetEvent::Modified { id } if *id == map_handle.id() => {
                    debug!("Map modified");
                }
                _ => (),
            }
        }
    }
}

/// Check to see if any maps' assets became available
/// if so.
pub fn update_loading_maps<UserData>(
    mut images: ResMut<Assets<Image>>,
    mut map_materials: ResMut<Assets<Map<UserData>>>,
    mut maps: Query<
        (
            Entity,
            Option<&MapAttributes>,
            &Handle<Map<UserData>>,
            Option<&MeshManagedByMap>,
        ),
        With<MapLoading>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    time: Res<Time>,
) where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    let images = images.as_mut();

    for (entity, attributes, map_handle, manage_mesh) in maps.iter_mut() {
        let Some(map) = map_materials.get_mut(map_handle) else {
            continue;
        };
        let Some(atlas) = images.get_mut(map.atlas_texture.clone()) else {
            continue;
        };

        atlas.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            // min_filter of linear gives undesired grid lines when zooming out
            min_filter: ImageFilterMode::Nearest,
            // mag_filter of linear gives mushy edges on tiles in closeup which is
            // usually not what we want
            mag_filter: ImageFilterMode::Nearest,
            mipmap_filter: ImageFilterMode::Linear,
            ..default()
        });

        if map.map_uniform.n_pattern_indices > 0 {
            let Some(pattern_atlas) = images.get_mut(map.pattern_atlas_texture.clone()) else { continue; };
            pattern_atlas.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                min_filter: ImageFilterMode::Nearest,
                mag_filter: ImageFilterMode::Nearest,
                mipmap_filter: ImageFilterMode::Linear,
                ..default()
            });
        }

        commands.entity(entity).remove::<MapLoading>();
        map.update(images);

        if manage_mesh.is_some() {
            let mut mesh = Mesh::from(Rectangle {
                half_size: map.world_size() / 2.0,
            });

            MapAttributes::set_mix_color(attributes, &mut mesh);
            MapAttributes::set_map_position(attributes, &mut mesh, &map);
            MapAttributes::set_animation_state(attributes, &mut mesh, &time);

            let mesh = Mesh2dHandle(meshes.add(mesh));
            commands.entity(entity).insert(mesh);
        }

        debug!("Map loaded: {:?}", map.map_size());
    }
}

/// Update mesh if MapAttributes change
pub fn update_map_vertex_attributes<UserData>(
    map_materials: ResMut<Assets<Map<UserData>>>,
    maps: Query<(
        Entity,
        &Handle<Map<UserData>>,
        &MapAttributes,
        Option<&Mesh2dHandle>,
        Option<&MeshManagedByMap>,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    time: Res<Time>,
) where
    UserData:
        AsBindGroup + Reflect + Clone + Default + TypePath + ShaderType + WriteInto + ShaderSize,
{
    for (entity, map_handle, attr, mesh_handle, manage_mesh) in maps.iter() {
        let Some(map) = map_materials.get(map_handle) else {
            warn!("No map material");
            continue;
        };

        let mut mesh = if manage_mesh.is_some() {
            Mesh::from(Rectangle {
                half_size: map.world_size() / 2.0,
            })
        } else {
            meshes.get(mesh_handle.unwrap().0.clone()).unwrap().clone()
        };

        MapAttributes::set_mix_color(Some(attr), &mut mesh);
        MapAttributes::set_map_position(Some(attr), &mut mesh, &map);
        MapAttributes::set_animation_state(Some(attr), &mut mesh, &time);

        let mesh = Mesh2dHandle(meshes.add(mesh));
        commands.entity(entity).insert(mesh);
    }
}
