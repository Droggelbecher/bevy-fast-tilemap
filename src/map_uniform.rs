use bevy::math::ivec2;
use bevy::{math::vec2, prelude::*, render::render_resource::ShaderType};

#[derive(ShaderType, Clone, Default, Debug, Reflect)]
pub struct MapUniform {
    /// Size of the map, in tiles.
    /// Will be derived from underlying map texture.
    pub(crate) map_size: UVec2,

    /// Size of the tile atlas, in pixels.
    /// Will be derived from the tile atlas texture.
    pub(crate) atlas_size: Vec2,

    /// Size of each tile, in pixels.
    pub(crate) tile_size: Vec2,

    /// Padding between tiles in atlas.
    pub(crate) tile_padding: Vec2,

    /// Relative anchor point position in a tile (in [0..1]^2)
    pub(crate) tile_anchor_point: Vec2,

    /// Number of paddings of size `tile_padding` at the top
    /// and left of the tilemap atlas. Eg (0, 0) or (1, 1).
    pub(crate) tile_paddings_topleft: UVec2,

    /// Number of paddings of size `tile_padding` at the bottom
    /// and right of the tilemap atlas. Eg (0, 0) or (1, 1).
    pub(crate) tile_paddings_bottomright: UVec2,

    /// fractional 2d map index -> world pos
    pub(crate) projection: Mat2,

    // -----
    /// [derived] Size of the map in world units necessary to display
    /// all tiles according to projection.
    pub(crate) world_size: Vec2,

    /// [derived] Offset of the projected map in world coordinates
    pub(crate) world_offset: Vec2,

    /// [derived]
    pub(crate) n_tiles: UVec2,

    /// [derived] world pos -> fractional 2d map index
    pub(crate) inverse_projection: Mat2,

    // ShaderType doesnt handle bools very well
    pub(crate) ready: u32,
}

impl MapUniform {
    pub(crate) fn map_size(&self) -> UVec2 {
        self.map_size
    }

    pub(crate) fn world_size(&self) -> Vec2 {
        self.world_size
    }

    pub(crate) fn map_to_world(&self, map_position: Vec2) -> Vec2 {
        (self.projection * map_position) * self.tile_size + self.world_offset
    }

    pub(crate) fn world_to_map(&self, world: Vec2) -> Vec2 {
        self.inverse_projection * ((world - self.world_offset) / self.tile_size)
    }

    pub(crate) fn ready(&self) -> bool {
        self.ready != 0u32
    }

    pub(crate) fn set_dirty(&mut self) {
        self.ready = 0u32
    }

    pub(crate) fn needs_update(&self, map_size: UVec2, atlas_size: Vec2) -> bool {
        !self.ready() || self.map_size != map_size || self.atlas_size != atlas_size
    }

    /// Return true iff this update made the uniform ready
    /// (ie. it was not ready before and is ready now).
    pub(crate) fn update(&mut self, map_size: UVec2, atlas_size: Vec2) -> bool {
        if !self.needs_update(map_size, atlas_size) {
            return false;
        }

        self.map_size = map_size;
        self.atlas_size = atlas_size;

        // World Size
        //
        // Determine the bounding rectangle of the projected map (in order to construct the quad
        // that will hold the texture).
        //
        // There is probably a more elegant way to do this, but this
        // works and is simple enough:
        // 1. save coordinates for all 4 corners
        // 2. take maximum x- and y distances
        let mut low = self.map_to_world(vec2(0.0, 0.0));
        let mut high = low.clone();
        for corner in [
            vec2(self.map_size().x as f32, 0.0),
            vec2(0.0, self.map_size().y as f32),
            vec2(self.map_size().x as f32, self.map_size().y as f32),
        ] {
            let pos = self.map_to_world(corner);
            low = low.min(pos);
            high = high.max(pos);
        }
        self.world_size = high - low;

        // World offset
        //
        // `map.projection` keeps the map coordinate (0, 0) at the world coordinate (0, 0).
        // However after projection we may want the (0, 0) tile to map to a different position than
        // say the top left corner (eg for an iso projection it might be vertically centered).
        // We use `low` from above to figure out how to correctly translate here.
        self.world_offset = vec2(-0.5, -0.5) * self.world_size - low;

        self.inverse_projection = self.projection.inverse();

        self.n_tiles = self.compute_n_tiles();

        self.ready = 1u32;
        true
    }

    fn compute_n_tiles(&self) -> UVec2 {
        // Consider x dimension: (y analog):
        // - Texture has width w (pixels)
        // - We have k (0..2) paddings on the left/right side of the atlas
        //    They have width p
        // - We have n tiles and (n-1) paddings in the "inner" part.
        // - Tiles have width t
        //
        // (k + n - 1) * p + n * t == w
        // we want to determine n
        //    p*k + p*n - p + n * t
        // == n * (p + t) + p * (k - 1) == w
        // <=> n = (w - p * (k - 1)) / (p + t)

        let n_outside_paddings =
            (self.tile_paddings_topleft + self.tile_paddings_bottomright).as_ivec2();

        let padding_offset = (n_outside_paddings - ivec2(1, 1)).as_vec2() * self.tile_padding;

        let n_tiles = (self.atlas_size - padding_offset) / (self.tile_padding + self.tile_size);

        let eps = 0.01;
        if (n_tiles.x - n_tiles.x.floor()).abs() > eps
            || (n_tiles.y - n_tiles.y.floor()).abs() > eps
        {
            panic!(
                "Expected an integral number of tiles in your atlas, but computes to be {:?}",
                n_tiles
            );
        }
        println!("n_tiles={n_tiles:?}");
        n_tiles.as_uvec2()
    }
}
