use bevy::render::render_resource::ShaderDefVal;
use bevy::{
    math::{mat2, vec2, vec3, Vec3Swizzles},
    prelude::*,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, ShaderRef, VertexFormat},
        texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    },
    sprite::{Material2d, Mesh2dHandle},
};

use crate::{map_builder::MapBuilder, map_uniform::MapUniform, shader::SHADER_HANDLE};

const ATTRIBUTE_MIX_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("MixColor", 988779055, VertexFormat::Float32x4);
const ATTRIBUTE_MIX_LEVEL: MeshVertexAttribute =
    MeshVertexAttribute::new("MixLevel", 988779056, VertexFormat::Float32);

/// Map, holding handles to a map texture with the tile data and an atlas texture
/// with the tile renderings.
#[derive(Asset, Debug, Clone, Default, Reflect, AsBindGroup)]
#[bind_group_data(MapKey)]
pub struct Map {
    /// Stores all the data that goes into the shader uniform,
    /// such as projection data, offsets, sizes, etc..
    #[uniform(0)]
    pub(crate) map_uniform: MapUniform,

    /// Texture containing the tile IDs (one per each pixel)
    #[storage(100, read_only)]
    pub(crate) map_texture: Vec<u32>,

    /// Atlas texture with the individual tiles
    #[texture(101)]
    #[sampler(102)]
    pub(crate) atlas_texture: Handle<Image>,

    pub(crate) perspective_defs: Vec<String>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct MapKey {
    pub(crate) perspective_defs: Vec<String>,
}

impl From<&Map> for MapKey {
    fn from(map: &Map) -> Self {
        MapKey {
            perspective_defs: map.perspective_defs.clone(),
        }
    }
}

/// Per-vertex attributes for map
#[derive(Component, Default, Clone, Debug)]
pub struct MapAttributes {
    pub mix_color: Vec<Vec4>,
    pub mix_level: Vec<f32>,
}

impl Material2d for Map {
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
            ATTRIBUTE_MIX_COLOR.at_shader_location(1),
            ATTRIBUTE_MIX_LEVEL.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];

        let fragment = descriptor.fragment.as_mut().unwrap();

        for def in key.bind_group_data.perspective_defs.iter() {
            fragment.shader_defs.push(ShaderDefVal::Bool(def.clone(), true));
        }

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

impl Map {
    /// Create a [`MapBuilder`] for configuring your map.
    pub fn builder(map_size: UVec2, atlas_texture: Handle<Image>, tile_size: Vec2) -> MapBuilder {
        MapBuilder::new(map_size, atlas_texture, tile_size)
    }

    pub fn indexer_mut(&mut self) -> MapIndexer {
        MapIndexer { map: self }
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
        let atlas_texture = match images.get(&self.atlas_texture) {
            Some(x) => x,
            None => {
                warn!("No atlas texture");
                return false;
            }
        };

        self.map_uniform
            .update_atlas_size(atlas_texture.size().as_vec2())
    }

    pub(crate) fn update_inverse_projection(&mut self) {
        self.map_uniform.inverse_projection =
            mat2(self.map_uniform.projection.x_axis.xy(), self.map_uniform.projection.y_axis.xy()).inverse();

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
        for (offset, def) in offsets.iter() {
            if self.map_uniform.map_to_local(offset.extend(0.0)).z < 0.0 {
                defs.push(format!("PERSPECTIVE_UNDER_{}", def));
            }
        }
        self.perspective_defs = defs;
    }

} // impl Map

// Indexer into a map.
// Internally holds a mutable reference to the underlying texture.
// See [`Map::get_mut`] for a usage example.
#[derive(Debug)]
pub struct MapIndexer<'a> {
    pub(crate) map: &'a mut Map,
}

impl<'a> MapIndexer<'a> {
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
}

///
pub fn configure_loaded_assets(
    map_materials: ResMut<Assets<Map>>,
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    map_handles: Query<&Handle<Map>>,
) {
    for ev in ev_asset.read() {
        for map_handle in map_handles.iter() {
            let Some(map) = map_materials.get(map_handle) else {
                warn!("No map material");
                continue;
            };

            match ev {
                AssetEvent::Added { id } if *id == map.atlas_texture.id() => {
                    // Set some sampling options for the atlas texture for nicer looks,
                    // such as avoiding "grid lines" when zooming out or mushy edges.
                    //
                    if let Some(atlas) = images.get_mut(&map.atlas_texture) {
                        // the below seems to crash?
                        //atlas.texture_descriptor.mip_level_count = 3;
                        atlas.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            // min_filter of linear gives undesired grid lines when zooming out
                            min_filter: ImageFilterMode::Nearest,
                            // mag_filter of linear gives mushy edges on tiles in closeup which is
                            // usually not what we want
                            mag_filter: ImageFilterMode::Nearest,
                            mipmap_filter: ImageFilterMode::Linear,
                            ..default()
                        });

                        if let Some(ref mut view_descriptor) = atlas.texture_view_descriptor {
                            view_descriptor.mip_level_count = Some(4);
                        }
                    }
                }
                _ => (),
            } // match ev
        } // for map
    } // for ev
} // configure_loaded_assets()

pub fn log_map_events(
    mut ev_asset: EventReader<AssetEvent<Map>>,
    map_handles: Query<&Handle<Map>>,
) {
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
pub fn update_loading_maps(
    images: Res<Assets<Image>>,
    mut map_materials: ResMut<Assets<Map>>,
    mut maps: Query<
        (
            Entity,
            Option<&MapAttributes>,
            &Handle<Map>,
            Option<&MeshManagedByMap>,
        ),
        With<MapLoading>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (entity, attributes, map_handle, manage_mesh) in maps.iter_mut() {
        let Some(map) = map_materials.get_mut(map_handle) else {
            warn!("No map material");
            continue;
        };

        commands.entity(entity).remove::<MapLoading>();
        map.update(images.as_ref());

        if manage_mesh.is_some() {
            debug!("Adding mesh for {entity:?}");

            let mut mesh = Mesh::from(shape::Quad {
                size: map.world_size(),
                flip: false,
            });

            if let Some(attr) = attributes {
                mesh = mesh
                    .with_inserted_attribute(ATTRIBUTE_MIX_COLOR, attr.mix_color.clone())
                    .with_inserted_attribute(ATTRIBUTE_MIX_LEVEL, attr.mix_level.clone());
            }

            let mesh = Mesh2dHandle(meshes.add(mesh));
            commands.entity(entity).insert(mesh);
        }

        debug!("Map loaded: {:?}", map.map_size());
    }
}

/// Update mesh if MapAttributes change
pub fn update_map_vertex_attributes(
    map_materials: ResMut<Assets<Map>>,
    maps: Query<(Entity, &Handle<Map>, &MapAttributes), Changed<MapAttributes>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (entity, map_handle, attr) in maps.iter() {
        let Some(map) = map_materials.get(map_handle) else {
            warn!("No map material");
            continue;
        };

        let mut mesh = Mesh::from(shape::Quad {
            size: map.world_size(),
            flip: false,
        });

        mesh = mesh
            .with_inserted_attribute(ATTRIBUTE_MIX_COLOR, attr.mix_color.clone())
            .with_inserted_attribute(ATTRIBUTE_MIX_LEVEL, attr.mix_level.clone());
        let mesh = Mesh2dHandle(meshes.add(mesh));
        commands.entity(entity).insert(mesh);
    }
}

pub fn apply_map_transforms(
    mut maps: Query<(&Handle<Map>, &GlobalTransform), Changed<GlobalTransform>>,
    mut map_materials: ResMut<Assets<Map>>,
) {
    for (map_handle, transform) in &mut maps {
        let Some(map) = map_materials.get_mut(map_handle) else {
            warn!("No map material");
            continue;
        };
        map.map_uniform.apply_transform(transform.clone());
    }
}
