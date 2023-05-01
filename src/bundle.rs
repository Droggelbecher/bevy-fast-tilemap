use crate::map::{Map, MapIndexer, MapDirty};
use crate::map_uniform::MapUniform;
use bevy::{
    math::{mat2, uvec2, vec2},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    sprite::Mesh2dHandle,
};
use std::mem::size_of;

/// Descriptor for creating a `MapBundle`.
/// Implements `Default` for convenience,
/// however you should really define at least:
/// - `map_size`
/// - `tile_size`
/// - `atlas_texture`
///
/// For non-rectangular map-tiles check out setting a different
/// `projection` such as `AXONOMETRIC`.
pub struct MapDescriptor {
    /// Size of the map (in tiles)
    pub map_size: UVec2,

    /// Size of a single tile (in pixels)
    pub tile_size: Vec2,

    /// Gap between tiles in tile atlas
    pub tile_padding: Vec2,

    /// Images holding the texture "atlas".
    pub atlas_texture: Handle<Image>,

    /// Transform of the quad holding the tilemap
    pub transform: Transform,

    /// Projection of the tilemap. Usually you want either
    /// IDENTITY or AXONOMETRIC
    pub projection: TileProjection,
}

#[derive(Debug, Clone, Copy)]
pub struct TileProjection {
    /// Projection matrix for converting map coordinates to world coordinates
    pub projection: Mat2,

    /// Relative anchor point into a tile.
    /// `(0.0, 0.0)` is top left, `(1.0, 1.0)` is bottom-right
    pub tile_anchor_point: Vec2,
}

/// Default projection that renders every tile as-is in a rectangular grid.
pub const IDENTITY: TileProjection = TileProjection {
    // By default flip y so tiles are rendered right side up
    projection: mat2(vec2(1.0, 0.0), vec2(0.0, -1.0)),
    tile_anchor_point: vec2(0.0, 0.0),
};

/// Assumes the tiles reperesent projections of square tiles
/// in an axonometric (eg isometric) projection,
/// i.e. tiles are diamond-shaped,
/// the origin is at center-left,
/// X-axis goes from origin down to bottom-center
/// Y-axis goes from origin up to top-center
pub const AXONOMETRIC: TileProjection = TileProjection {
    /*
     *         __--X--__
     *    __---         ---__
     * A--__               __---
     *      ---__     __---
     *           --Y--
     *
     * (A) anchor point, vertically centered i.e. at (0.0, 0.5)
     * in relative tile cooridinates
     *
     * X-axis goes from (A) to (X), i.e. in map coordinates (A)
     * is at (0, 0) (by definition, its the anchor point),
     * and (X) is at (1, 0) (as defined by `projection`).
     * Analogously, Y is at (0, 1) in map coordinates.
     *
     */
    projection: Mat2::from_cols(vec2(0.5, -0.5), vec2(0.5, 0.5)),
    tile_anchor_point: vec2(0.0, 0.5),
};

impl Default for MapDescriptor {
    fn default() -> Self {
        Self {
            map_size: uvec2(100, 100),
            tile_size: vec2(16.0, 16.0),
            tile_padding: vec2(0.0, 0.0),
            atlas_texture: default(),
            transform: default(),
            projection: IDENTITY,
        }
    }
}

impl MapDescriptor {
    /// Build map bundle with default initialization (index 0).
    pub fn build(
        self,
        images: &mut ResMut<Assets<Image>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> MapBundle {
        self.build_and_initialize(images, meshes, |_| {})
    }

    /// `initializer' will be called once to initialize the map data using the given MapIndexer.
    /// Prior to calling this the map will be initialized with zeros.
    pub fn build_and_initialize<F>(
        self,
        images: &mut ResMut<Assets<Image>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        initializer: F,
    ) -> MapBundle
    where
        F: FnOnce(&mut MapIndexer) -> (),
    {
        let mut map_image = Image::new(
            Extent3d {
                width: self.map_size.x as u32,
                height: self.map_size.y as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![0u8; (self.map_size.x * self.map_size.y) as usize * size_of::<u16>()],
            TextureFormat::R16Uint,
        );
        map_image.texture_descriptor.usage = TextureUsages::STORAGE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::TEXTURE_BINDING;
        map_image.texture_descriptor.mip_level_count = 1;

        initializer(&mut MapIndexer {
            image: &mut map_image,
            size: self.map_size,
        });

        let projection = self.projection.projection;

        let map = Map {
            map_uniform: MapUniform {
                map_size: self.map_size,
                tile_size: self.tile_size,
                inner_padding: self.tile_padding,
                projection,
                tile_anchor_point: self.projection.tile_anchor_point,
                ..default()
            },
            map_texture: images.add(map_image),
            atlas_texture: self.atlas_texture.clone(),
        };

        // TODO:
        // Reconsider mesh resizing. Perhaps cleanest approach:
        // - Control with a (descriptor/map) flag whether this map should try to resize its mesh
        // - If off, generate mesh of some known default size
        // - User can provide their own mesh in the bundle like MapBundle { mesh: my_mesh,
        // ..MapDescriptor{...}.build() } or by replacing
        // - If on, update mesh in a system
        //
        // See bevy_render/src/mesh/shape/mod.rs
        // will generate 3d position, 3d normal, and 2d UVs
        //
        // TODO: Make an example that shows off the custom mesh, perhaps 3d.
        // update feature list
        let mesh = Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
            size: vec2(1000.0, 1000.0),
            flip: false,
        })));

        // TODO: Consider turning this into constructing map only
        // and have the user do the rest as its sufficiently simple now
        MapBundle {
            map,
            mesh: mesh.clone(),
            transform: self.transform,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
            dirty: MapDirty::default(),
        }

        //commands.spawn(bundle)
    } // fn spawn()
} // impl MapDescriptor

#[derive(Bundle, Clone)]
pub struct MapBundle {
    pub mesh: Mesh2dHandle,
    pub map: Map,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub dirty: MapDirty,
}
