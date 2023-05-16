use crate::map::{Map, MapIndexer};
use crate::map_uniform::MapUniform;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use std::mem::size_of;

use crate::tile_projection::TileProjection;

/// Builder for constructing a map component. This is usually the preferred way of constructing.
pub struct MapBuilder {
    map: Map,
}

impl MapBuilder {
    /// Create a builder for the given map size (number of tiles in each dimension),
    /// the given atlas texture and the tile size (in the atlas).
    pub fn new(map_size: UVec2, atlas_texture: Handle<Image>, tile_size: Vec2) -> Self {
        Self {
            map: Map {
                atlas_texture,
                map_uniform: MapUniform {
                    map_size,
                    tile_size,
                    ..default()
                },
                ..default()
            },
        }
    } // fn new

    /// Us the given map projection for rendering. Default is [`crate::tile_projection::IDENTITY`],
    /// which will render the tiles in rectangular layout.
    pub fn with_projection(mut self, projection: TileProjection) -> Self {
        self.map.map_uniform.projection = projection.projection;
        self.map.map_uniform.tile_anchor_point = projection.tile_anchor_point;
        self
    }

    /// Specify the padding in the `atlas_texture`.
    /// `inner`: Padding between the tiles,
    /// `topleft`: Padding to top and left of the tile atlas,
    /// `bottomright`: Padding to bottom and right of the atlas.
    ///
    /// Note that it is crucial that these values are precisely correct,
    /// we use them internally to determine how many tiles there are in the atlas in each
    /// direction, if that does not produce a number close to an integer,
    /// you will get a `panic` when the tile atlas is loaded.
    pub fn with_padding(mut self, inner: Vec2, topleft: Vec2, bottomright: Vec2) -> Self {
        self.map.map_uniform.inner_padding = inner;
        self.map.map_uniform.outer_padding_topleft = topleft;
        self.map.map_uniform.outer_padding_bottomright = bottomright;
        self
    }

    /// Render this map in "dominance" overhang mode.
    /// "Dominance" overhang draws the overlap of tiles depending on their index in the tile atlas.
    /// Tiles with higher index will be drawn on top of tiles with lower index.
    /// For this we draw in the "padding" area of the tile atlas.
    ///
    /// This requires each pixel to be computed once for every level higher than the current one
    /// and for every neighbor which can be a drastic performance hit.
    /// Therefore its a good idea to limit the number of levels looked upwards here.
    pub fn with_dominance_overhang(mut self, max_overhang_levels: u32) -> Self {
        self.map.map_uniform.overhang_mode = 0;
        self.map.map_uniform.max_overhang_levels = max_overhang_levels;
        self
    }

    /// Render this map in "perspective" overhang mode.
    /// "Perspective" overhang draws the overlap of tiles depending on their "depth" that is the
    /// y-axis of their world position (tiles higher up are considered further away).
    pub fn with_perspective_overhang(mut self) -> Self {
        self.map.map_uniform.overhang_mode = 1;
        self
    }

    /// Build the map component.
    pub fn build(self, images: &mut ResMut<Assets<Image>>) -> Map {
        self.build_and_initialize(images, |_| {})
    }

    /// Build the map component and immediately initialize the map
    /// data with the given initializer callback.
    pub fn build_and_initialize<F>(
        mut self,
        images: &mut ResMut<Assets<Image>>,
        initializer: F,
    ) -> Map
    where
        F: FnOnce(&mut MapIndexer) -> (),
    {
        let mut map_image = Image::new(
            Extent3d {
                width: self.map.map_size().x as u32,
                height: self.map.map_size().y as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![0u8; (self.map.map_size().x * self.map.map_size().y) as usize * size_of::<u16>()],
            TextureFormat::R16Uint,
        );
        map_image.texture_descriptor.usage = TextureUsages::STORAGE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::TEXTURE_BINDING;
        map_image.texture_descriptor.mip_level_count = 1;

        initializer(&mut MapIndexer {
            image: &mut map_image,
            size: self.map.map_uniform.map_size,
        });

        self.map.map_texture = images.add(map_image);
        self.map.map_uniform.update_inverse_projection();
        self.map.map_uniform.update_world_size();

        self.map
    } // fn build_and_initialize
}
