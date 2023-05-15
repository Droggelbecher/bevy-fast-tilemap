use crate::map::{Map, MapIndexer};
use crate::map_uniform::MapUniform;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use std::mem::size_of;

use crate::tile_projection::TileProjection;

pub struct MapBuilder {
    map: Map,
}

impl MapBuilder {
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

    pub fn with_projection(mut self, projection: TileProjection) -> Self {
        self.map.map_uniform.projection = projection.projection;
        self.map.map_uniform.tile_anchor_point = projection.tile_anchor_point;
        self
    }

    pub fn with_padding(mut self, inner: Vec2, topleft: Vec2, bottomright: Vec2) -> Self {
        self.map.map_uniform.inner_padding = inner;
        self.map.map_uniform.outer_padding_topleft = topleft;
        self.map.map_uniform.outer_padding_bottomright = bottomright;
        self
    }

    pub fn with_max_overhang_levels(mut self, max_overhang_levels: u32) -> Self {
        self.map.map_uniform.max_overhang_levels = max_overhang_levels;
        self
    }

    pub fn build(self, images: &mut ResMut<Assets<Image>>) -> Map {
        self.build_and_initialize(images, |_| {})
    }

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
