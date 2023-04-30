use crate::map::{Map, MapIndexer, MapData};
use bevy::{
    math::{uvec2, mat2, vec2},
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
/// - `tiles_texture`
///
/// For non-rectangular map-tiles check out setting a different
/// `projection` such as `AXONOMETRIC`.
pub struct MapDescriptor {
    /// Size of the map (in tiles)
    pub map_size: UVec2,

    /// Size of a single tile (in pixels)
    pub tile_size: Vec2,

    /// Images holding the texture atlases, one for each layer of the map.
    /// All atlases must have a tile size of `tile_size` and no padding.
    pub tiles_texture: Handle<Image>,

    /// Transform of the quad holding the tilemap
    pub transform: Transform,

    /// Projection of the tilemap. Usually you want either
    /// IDENTITY or AXONOMETRIC
    pub projection: TileProjection,
}

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

    projection: Mat2::from_cols(
        vec2(0.5, -0.5),
        vec2(0.5, 0.5)
    ),
    tile_anchor_point: vec2(0.0, 0.5),
};

impl Default for MapDescriptor {
    fn default() -> Self {
        Self {
            map_size: uvec2(100, 100),
            tile_size: vec2(16.0, 16.0),
            tiles_texture: default(),
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
        initializer: F
    ) -> MapBundle
        where F: FnOnce(&mut MapIndexer) -> ()
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
        let inverse_projection = projection.inverse();

        // In the first step we use a zero offset, it will be corrected later
        let world_offset = Vec2::default();

        let mut map = Map {
            map_data: MapData {
                size: self.map_size,
                tile_size: self.tile_size,
                projection,
                inverse_projection,
                world_offset,
                tile_anchor_point: self.projection.tile_anchor_point,
            },
            map_texture: images.add(map_image),
            tiles_texture: self.tiles_texture.clone(),
            ready: false,
        };

        // Determine the bounding rectangle of the projected map (in order to construct the quad
        // that will hold the texture).
        //
        // There is probably a more elegant way to do this, but this
        // works and is simple enough:
        // 1. save coordinates for all 4 corners
        // 2. take maximum x- and y distances

        let mut low = map.map_to_world(vec2(0.0, 0.0));
        let mut high = low.clone();
        for corner in [
            vec2(map.map_data.size.x as f32, 0.0),
            vec2(0.0, map.map_data.size.y as f32),
            vec2(map.map_data.size.x as f32, map.map_data.size.y as f32),
        ] {
            let pos = map.map_to_world(corner);
            low = low.min(pos);
            high = high.max(pos);
        }
        let size = high - low;

        // `map.projection` keeps the map coordinate (0, 0) at the world coordinate (0, 0).
        // However after projection we may want the (0, 0) tile to map to a different position than
        // say the top left corner (eg for an iso projection it might be vertically centered).
        // We use `low` from above to figure out how to correctly translate here.

        map.map_data.world_offset = vec2(-0.5, -0.5) * size - low;

        // See bevy_render/src/mesh/shape/mod.rs
        // will generate 3d position, 3d normal, and 2d UVs
        let mesh = Mesh2dHandle(meshes.add(Mesh::from(shape::Quad { size, flip: false })));

        MapBundle {
            map,
            mesh: mesh.clone(),
            transform: self.transform,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
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
}
