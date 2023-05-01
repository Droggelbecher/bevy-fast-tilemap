
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
    tile_padding: vec2<f32>,

    /// Relative anchor point position in a tile (in [0..1]^2)
    tile_anchor_point: vec2<f32>,

    /// Number of paddings of size `tile_padding` at the top
    /// and left of the tilemap atlas. Eg (0, 0) or (1, 1).
    tile_paddings_topleft: vec2<u32>,

    /// Number of paddings of size `tile_padding` at the bottom
    /// and right of the tilemap atlas. Eg (0, 0) or (1, 1).
    tile_paddings_bottomright: vec2<u32>,

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
var tiles_texture: texture_2d<f32>;

@group(1) @binding(2)
var tiles_sampler: sampler;

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

// Custom vertex shader for passing along the UV coordinate
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

    var pos = index2d * (map.tile_size + map.tile_padding)
        + vec2<f32>(map.tile_paddings_topleft) * map.tile_padding;
    return pos;
}


@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    /*return vec4<f32>(1.0, 0.0, 0.0, 1.0);*/

    var map_pos = world_to_map(map, in.world_position.xy);

    // Integer part of map position (tile coordinate)
    var map_coord = floor(map_pos.xy);

    // fractional part (position inside tile)
    var world_tile_base = map_to_world(map, map_coord);

    var offset_world_coords = in.world_position.xy - world_tile_base;
    var offset = vec2<f32>(1.0, -1.0) * offset_world_coords;

    // tilemap index for that tile map_coord
    var index = u32(textureLoad(map_texture, vec2<i32>(map_coord)).r);

    var tile_start = atlas_index_to_position(map, index);
    return textureSample(
        tiles_texture, tiles_sampler,
        (tile_start + offset + map.tile_anchor_point * map.tile_size) / map.atlas_size
    );
}
