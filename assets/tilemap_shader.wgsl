
#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils
#import bevy_sprite::mesh2d_bindings

struct Map {
    /// Size of the map, in tiles.
    /// Will be derived from underlying map texture.
    map_size: vec2<u32>,

    /// Size of the tile atlas, in pixels.
    /// Will be derived from the tile atlas texture.
    atlas_size: vec2<f32>,

    /// Size of each tile, in pixels.
    tile_size: vec2<f32>,

    /// Padding between tiles in atlas.
    inner_padding: vec2<f32>,

    /// Padding at atlas top/left and bottom/right
    outer_padding_topleft: vec2<f32>,
    outer_padding_bottomright: vec2<f32>,

    /// Relative anchor point position in a tile (in [0..1]^2)
    tile_anchor_point: vec2<f32>,

    /// fractional 2d map index -> world pos
    projection: mat2x2<f32>,

    // -----
    /// [derived] Size of the map in world units necessary to display
    /// all tiles according to projection.
    world_size: vec2<f32>,

    /// [derived] Offset of the projected map in world coordinates
    world_offset: vec2<f32>,

    /// [derived]
    n_tiles: vec2<u32>,

    /// [derived] world pos -> fractional 2d map index
    inverse_projection: mat2x2<f32>,

    // ShaderType doesnt handle bools very well
    ready: u32,
};

@group(1) @binding(0)
var map_texture: texture_storage_2d<r16uint, read>;

@group(1) @binding(1)
var atlas_texture: texture_2d<f32>;

@group(1) @binding(2)
var atlas_sampler: sampler;

@group(1) @binding(3)
var<uniform> map: Map;

#import bevy_sprite::mesh2d_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
};

/// Custom vertex shader for passing along the UV coordinate
@vertex
fn vertex(v: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh2d_position_local_to_clip(mesh.model, vec4<f32>(v.position, 1.0));
    out.world_position = mesh2d_position_local_to_world(mesh.model, vec4<f32>(v.position, 1.0));
    return out;
}

/// Map position incl fractional part for this position.
fn world_to_map(map: Map, world_position: vec2<f32>) -> vec2<f32> {
    return (
        map.inverse_projection
        * ((world_position - map.world_offset) / map.tile_size)
    );
}

fn map_to_world(map: Map, map_position: vec2<f32>) -> vec2<f32> {
    return (
        (map.projection * map_position) * map.tile_size + map.world_offset
    );
}

/// Position (world/pixel units) in tilemap atlas of the top left corner
/// of the tile with the given index
fn atlas_index_to_position(map: Map, index: u32) -> vec2<f32> {
    var index_f = f32(index);
    var index_y = floor(index_f / f32(map.n_tiles.x));
    var index_x = index_f - index_y * f32(map.n_tiles.x);
    var index2d = vec2<f32>(index_x, index_y);

    var pos = index2d * (map.tile_size + map.inner_padding) + map.outer_padding_topleft;
    return pos;
}

/// Compute offset into the tile by world position and world position of tile reference point
fn world_to_tile_offset(world_position: vec2<f32>, world_tile_base: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(1.0, -1.0) * (world_position - world_tile_base);
}

/// Sample tile from the tile atlas
fn sample_tile(
    map: Map,
    tile_index: u32,
    tile_offset: vec2<f32>,
) -> vec4<f32> {
    var tile_start = atlas_index_to_position(map, tile_index);
    return textureSample(
        atlas_texture, atlas_sampler,
        (tile_start + tile_offset + map.tile_anchor_point * map.tile_size) / map.atlas_size
    );
}

struct MapPosition {
    // The 2d tile position on the map
    tile: vec2<i32>,
    // Offset in pixels/world coordinates from the reference position of that tile
    offset: vec2<f32>
};


/// Figure out where in the map (tile position & offset) this world position is.
fn world_to_tile_and_offset(
    world_position: vec2<f32>
) -> MapPosition {
    var out: MapPosition;

    // Map position including fractional part
    var pos = world_to_map(map, world_position);

    // Integer part of map position (tile coordinate)
    var tile = floor(pos);
    out.tile = vec2<i32>(tile);

    // World position of tile reference point
    var world_tile_base = map_to_world(map, tile);
    out.offset = world_to_tile_offset(world_position, world_tile_base);

    return out;
}

///
fn get_tile_index(map_position: vec2<i32>) -> u32 {
    return u32(textureLoad(map_texture, map_position).r);
}

fn blend(c0: vec4<f32>, c1: vec4<f32>) -> vec4<f32> {
    return mix(c0, c1, c1.a);
}

fn sample_neighbor(pos: MapPosition, tile_offset: vec2<i32>) -> vec4<f32> {
    // integral position of the neighbouring tile
    var tile = pos.tile + tile_offset;

    // kind of tile being displayed at that position
    var tile_index = get_tile_index(tile);

    var offset = (map.projection * vec2<f32>(-tile_offset)) * map.tile_size;

    return sample_tile(map, tile_index, pos.offset + vec2<f32>(1.0, -1.0) * offset);
}

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    var world_position = in.world_position.xy;

    var pos = world_to_tile_and_offset(world_position);
    var index = get_tile_index(pos.tile);
    var color = sample_tile(map, index, pos.offset);

    /*var top_tile = pos.tile + vec2<i32>(-1, 1);*/
    /*var top_tile_index = get_tile_index(top_tile);*/
    // TODO: should use actually inverse transform of (-1,1) here
    /*var top_tile_color = sample_tile(map, index, pos.offset + vec2<f32>(0.0, map.tile_size.y));*/

    // "top" tile (iso view)
    color = blend(color, sample_neighbor(pos, vec2<i32>(-1, 1)));

    // "top/left" tile (iso view)
    color = blend(color, sample_neighbor(pos, vec2<i32>(-1, 0)));

    // "top/right" tile (iso view)
    color = blend(color, sample_neighbor(pos, vec2<i32>(0, 1)));

    return color;
}


