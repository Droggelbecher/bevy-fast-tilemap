use bevy::{
    math::{mat3, vec2, vec3, Mat3},
    prelude::*,
};

/// Determines how map coordinates are related to world coordinates.
#[derive(Debug, Clone, Copy)]
pub struct TileProjection {
    /// Projection matrix for converting map coordinates to world coordinates.
    /// This is normalized to the tile dimensions, ie. 1.0 means full tile width/height.
    pub projection: Mat3,

    /// Relative anchor point into a tile.
    /// `(0.0, 0.0)` is top left, `(1.0, 1.0)` is bottom-right
    pub tile_anchor_point: Vec2,
}

/// Default projection that renders every tile as-is in a rectangular grid.
pub const IDENTITY: TileProjection = TileProjection {
    // By default flip y so tiles are rendered right side up
    projection: mat3(
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
    ),
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
    projection: mat3(
        vec3(0.5, -0.5, 0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(0.0, -1.0, 0.0),
    ),

    tile_anchor_point: vec2(0.0, 0.5),
};
