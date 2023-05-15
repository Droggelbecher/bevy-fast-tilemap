use bevy::{
    math::{vec2, Vec3Swizzles, vec3, mat2},
    prelude::*,
    render::render_resource::ShaderType,
};

use crate::tile_projection::IDENTITY;

#[derive(ShaderType, Clone, Debug, Reflect)]
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
    pub(crate) inner_padding: Vec2,

    /// Padding at atlas top/left and bottom/right
    pub(crate) outer_padding_topleft: Vec2,
    pub(crate) outer_padding_bottomright: Vec2,

    /// Relative anchor point position in a tile (in [0..1]^2)
    pub(crate) tile_anchor_point: Vec2,

    /// fractional 2d map index -> world pos
    pub(crate) projection: Mat3,

    /// 0=dominance
    /// 1=perspective
    pub(crate) overhang_mode: u32,

    /// For overhang_mode==0
    pub(crate) max_overhang_levels: u32,

    /// For overhang_mode==1
    pub(crate) perspective_overhang_mask: u32,

    // -----
    /// [derived] Size of the map in world units necessary to display
    /// all tiles according to projection.
    pub(crate) world_size: Vec2,

    /// [derived] Offset of the projected map in world coordinates
    pub(crate) world_offset: Vec2,

    /// [derived]
    pub(crate) n_tiles: UVec2,

    /// [derived] world pos -> fractional 2d map index
    ///
    /// Note that the main use case for the inverse is to transform 2d world coordinates 
    /// (eg from mouse cursor) to 2d map coordinates with some assumption about how we choose the z
    /// coordinate.
    /// An inverse of the 3d projection matrix here would assume that you feed in the correct z
    /// coordinate and otherwise give wrong results, hence we only invert the 2d part
    /// and let the caller handle management of z.
    pub(crate) inverse_projection: Mat2,
}

impl Default for MapUniform {
    fn default() -> Self {
        Self {
            map_size: default(),
            atlas_size: default(),
            tile_size: default(),
            inner_padding: default(),
            outer_padding_topleft: default(),
            outer_padding_bottomright: default(),
            tile_anchor_point: IDENTITY.tile_anchor_point,
            projection: IDENTITY.projection,
            overhang_mode: default(),
            max_overhang_levels: default(),
            perspective_overhang_mask: default(),
            world_size: default(),
            world_offset: default(),
            n_tiles: default(),
            inverse_projection: default(),
        }
    }
}

impl MapUniform {
    pub(crate) fn map_size(&self) -> UVec2 {
        self.map_size
    }

    pub(crate) fn world_size(&self) -> Vec2 {
        self.world_size
    }

    pub(crate) fn map_to_world(&self, map_position: Vec3) -> Vec3 {
        (self.projection * map_position) * self.tile_size.extend(1.0) + self.world_offset.extend(0.0)
    }

    /// As of now, this will ignore `world`s z coordinate
    /// and always project to z=0 on the map.
    /// This behavior might change in the future
    pub(crate) fn world_to_map(&self, world: Vec3) -> Vec3 {
        (self.inverse_projection * ((world.xy() - self.world_offset) / self.tile_size)).extend(0.0)
    }

    pub(crate) fn update_map_size(&mut self, map_size: UVec2) -> bool {
        if self.map_size == map_size {
            return false;
        }

        self.map_size = map_size;
        self.update_world_size();
        true
    }

    pub(crate) fn update_world_size(&mut self) {
        // World Size
        //
        // Determine the bounding rectangle of the projected map (in order to construct the quad
        // that will hold the texture).
        //
        // There is probably a more elegant way to do this, but this
        // works and is simple enough:
        // 1. save coordinates for all 4 corners
        // 2. take maximum x- and y distances
        let mut low = self.map_to_world(vec3(0.0, 0.0, 0.0)).xy();
        let mut high = low.clone();
        for corner in [
            vec2(self.map_size().x as f32, 0.0),
            vec2(0.0, self.map_size().y as f32),
            vec2(self.map_size().x as f32, self.map_size().y as f32),
        ] {
            let pos = self.map_to_world(corner.extend(0.0)).xy();
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
    }

    /// Return true iff this update made the uniform ready
    /// (ie. it was not ready before and is ready now).
    pub(crate) fn update_atlas_size(&mut self, atlas_size: Vec2) -> bool {
        if self.atlas_size == atlas_size {
            return false;
        }

        self.atlas_size = atlas_size;
        self.update_n_tiles();
        true
    }

    pub(crate) fn update_inverse_projection(&mut self) {
        self.inverse_projection = mat2(
            self.projection.x_axis.xy(),
            self.projection.y_axis.xy(),
        ).inverse();
    }

    fn update_n_tiles(&mut self) {
        let inner = self.atlas_size - self.outer_padding_topleft - self.outer_padding_bottomright;
        let n_tiles = (inner + self.inner_padding) / (self.inner_padding + self.tile_size);

        let eps = 0.01;
        if (n_tiles.x - n_tiles.x.round()).abs() > eps
            || (n_tiles.y - n_tiles.y.round()).abs() > eps
        {
            panic!(
                "Expected an integral number of tiles in your atlas, but computes to be {:?}",
                n_tiles
            );
        }
        self.n_tiles = n_tiles.as_uvec2();
    }
}
