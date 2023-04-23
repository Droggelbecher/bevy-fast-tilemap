
#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils
#import bevy_sprite::mesh2d_bindings

struct Map {
    // Number of tiles in the tilemap in each dimension
    tilemap_tiles: vec2<f32>,
    // Size of inidividual tile
    tile_size: vec2<f32>,

    // fractional 2d map index -> world pos
    projection: mat2x2<f32>,

    // world pos -> fractional 2d map index
    inverse_projection: mat2x2<f32>,

    // Offset of the projected map in world coordinates
    world_offset: vec2<f32>,

    // relative anchor point position in a tile (in [0..1]^2)
    tile_anchor_point: vec2<f32>,
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

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {

    var map_size = textureDimensions(map_texture);

    // Map position incl fractional part for this position.
    var map_pos = map.inverse_projection * ((in.world_position.xy - map.world_offset) / map.tile_size);

    // Integer part of map position (tile coordinate)
    var map_coord = floor(map_pos.xy);

    // fractional part (position inside tile)
    var world_tile_base = (map.projection * map_coord.xy) * map.tile_size + map.world_offset;

    var offset_world_coords = in.world_position.xy - world_tile_base;
    var offset = vec2<f32>(1.0, -1.0) * offset_world_coords / map.tile_size;

    // tilemap index for that tile map_coord
    var index = f32(textureLoad(map_texture, vec2<i32>(map_coord)).r);

    // Convert index to x/y tile position in tilemap
    var index_y = floor(index / map.tilemap_tiles.x);
    var index_x = index - index_y * map.tilemap_tiles.x;

    // Sample the tilemap
    return textureSample(
        tiles_texture, tiles_sampler,
        (vec2<f32>(index_x, index_y) + offset + map.tile_anchor_point) / map.tilemap_tiles
    );
}
