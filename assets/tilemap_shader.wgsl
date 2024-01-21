#import bevy_sprite::mesh2d_bindings mesh

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

    /// fractional 2d map index -> relative local world pos
    projection: mat3x3<f32>,

    /// Global transform of the entity holding the map as transformation matrix & offset.
    /// This is currently redundant with mesh.model,
    /// which should represent the same info as a 4x4 affine matrix, but we consider it a bit
    /// more consistent in conjunction with the inverse below. May be removed in the future.
    global_transform_matrix: mat3x3<f32>,
    global_transform_translation: vec3<f32>,

    overhang_mode: u32,
    max_overhang_levels: u32,
    perspective_overhang_mask: u32, // TODO: Remove

    // -----
    /// [derived] Size of the map in world units necessary to display
    /// all tiles according to projection.
    world_size: vec2<f32>,

    /// [derived] Offset of the projected map in world coordinates
    world_offset: vec2<f32>,

    /// [derived]
    n_tiles: vec2<u32>,

    /// [derived] local world pos -> fractional 2d map index
    inverse_projection: mat2x2<f32>,

    /// [derived] Iverse of global transform of the entity holding the map as transformation matrix & offset.
    global_inverse_transform_matrix: mat3x3<f32>,
    global_inverse_transform_translation: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> map: Map;

@group(1) @binding(100)
//var map_texture: texture_storage_2d<r16uint, read>;
var<storage> map_texture: array<u32>;

@group(1) @binding(101)
var atlas_texture: texture_2d<f32>;

@group(1) @binding(102)
var atlas_sampler: sampler;

#import bevy_sprite::mesh2d_functions::{get_model_matrix, mesh2d_position_local_to_clip, mesh2d_position_local_to_world}
//#import bevy_sprite::mesh2d_functions mesh2d_position_local_to_clip, mesh2d_position_local_to_world
//#import bevy_sprite::mesh2d::mesh2d::Vertex
struct Vertex {
    @builtin(instance_index) instance_index: u32,
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
};
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

/*
struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
};
*/

/// Custom vertex shader for passing along the UV coordinate
@vertex
fn vertex(v: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var model: mat4x4<f32> = get_model_matrix(v.instance_index);

    out.position = mesh2d_position_local_to_clip(model, vec4<f32>(v.position, 1.0));
    out.world_position = mesh2d_position_local_to_world(model, vec4<f32>(v.position, 1.0));
    return out;
}

/// Map position incl fractional part for this position.
fn world_to_map(map: Map, world_position: vec2<f32>) -> vec2<f32> {
    // Steps:
    // - Apply inverse global transform
    // - Adjust for `map.world_offset` (where in the mesh tile 0,0 should be)
    // - Scale according to `map.tile_size`
    // - Apply inverse map projection for tile distortion (eg iso)
    var local_world_pos = map.global_inverse_transform_matrix * vec3<f32>(world_position, 0.0) + map.global_inverse_transform_translation;
    var pos = (local_world_pos.xy - map.world_offset) / map.tile_size;
    return map.inverse_projection * pos;
}

fn map_to_world(map: Map, map_position: vec2<f32>) -> vec3<f32> {
    // Steps:
    // - Apply map projection (to compensate for eg iso view)
    // - scale according to `map.tile_size`
    // - Adjust for `map.world_offset` (where in the mesh tile 0,0 should be)
    // - Apply global transform
    return map.global_transform_matrix * (
        (map.projection * vec3<f32>(map_position, 0.0)) * vec3<f32>(map.tile_size, 1.0) +
        vec3<f32>(map.world_offset, 0.0)
    ) + map.global_transform_translation;
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
/// tile_index: Index of the tile in the atlas
/// tile_offset: Offset from tile anchor point in pixel/world coordinates
fn sample_tile(
    map: Map,
    tile_index: u32,
    tile_offset: vec2<f32>,
) -> vec4<f32> {

    /*

      +-----------+    :    +------------+
      |           |    :    |            |
      a           |    :    |            |
      |           |    :    |            |
      +-----------+    :    +------------+
                       :
      .................:..................
                       :
      +-----------+    :    +------------+
      |           |    :    |            |
      |           |    :    |            |
      |           |    :    |            |
      +-----------+    :    +------------+

      [   TILE    ] Padding [    TILE    ]

    */

    var tile_start = atlas_index_to_position(map, tile_index);
    var rect_offset = tile_offset + map.tile_anchor_point * map.tile_size;
    var total_offset = tile_start + rect_offset;

    // At most half of the inner "padding" is still rendered
    // as overhang of any given tile.
    // Outer padding is not taken into account
    var max_overhang = map.inner_padding / 2.0;

    var color = textureSample(
        atlas_texture, atlas_sampler, total_offset / map.atlas_size
    );

    // Outside of "our" part of the padding, dont render anything as part of this tile,
    // as it might be used for overhang of a neighbouring tile in the tilemap
    if rect_offset.x < -max_overhang.x
        || rect_offset.y < -max_overhang.y
        || rect_offset.x > (map.tile_size.x + max_overhang.x)
        || rect_offset.y > (map.tile_size.y + max_overhang.y)
    {
        color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    return color;
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
    var world_tile_base = map_to_world(map, tile).xy;
    out.offset = world_to_tile_offset(world_position, world_tile_base);

    return out;
}

///
fn get_tile_index(map_position: vec2<i32>) -> u32 {
    //return u32(textureLoad(map_texture, map_position).r);
    return map_texture[map_position.y * i32(map.map_size.x) + map_position.x];
}

fn blend(c0: vec4<f32>, c1: vec4<f32>) -> vec4<f32> {
    return mix(c0, c1, c1.a);
}

fn is_valid_tile(map: Map, tile: vec2<i32>) -> bool {
    if tile.x < 0 || tile.y < 0 {
        return false;
    }
    let map_size = vec2<i32>(map.map_size);
    if tile.x >= map_size.x || tile.y >= map_size.y {
        return false;
    }
    return true;
}

fn sample_neighbor_tile_index(tile_index: u32, pos: MapPosition, tile_offset: vec2<i32>) -> vec4<f32> {
    var overhang = (map.projection * vec3<f32>(vec2<f32>(-tile_offset), 0.0)).xy * map.tile_size;
    var offset = pos.offset + vec2<f32>(1.0, -1.0) * overhang;
    return sample_tile(map, tile_index, offset);
}

fn sample_neighbor(pos: MapPosition, tile_offset: vec2<i32>) -> vec4<f32> {
    // integral position of the neighbouring tile
    var tile = pos.tile + tile_offset;
    if !is_valid_tile(map, tile) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // kind of tile being displayed at that position
    var tile_index = get_tile_index(tile);
    return sample_neighbor_tile_index(tile_index, pos, tile_offset);
}

fn sample_neighbor_if_ge(index: u32, pos: MapPosition, tile_offset: vec2<i32>) -> vec4<f32> {
    // integral position of the neighbouring tile
    var tile = pos.tile + tile_offset;
    if !is_valid_tile(map, tile) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // kind of tile being displayed at that position
    var tile_index = get_tile_index(tile);
    if tile_index >= index {
        return sample_neighbor_tile_index(tile_index, pos, tile_offset);
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

fn render_dominance_overhangs(color: vec4<f32>, index: u32, pos: MapPosition) -> vec4<f32> {
    var max_index = min(map.n_tiles.x * map.n_tiles.y, index + map.max_overhang_levels);
    var c = color;

    // Note: For some reason on OSX, the use of for loops fails silently (produces pure red output
    // in our case), while a loop { ... } seems to work just fine.
    var idx = index + u32(1);
    loop {
        if idx >= max_index { break; }

        // first render all the diagonal overhangs
        c = blend(c, sample_neighbor_if_ge(idx, pos, vec2<i32>(-1, -1)));
        c = blend(c, sample_neighbor_if_ge(idx, pos, vec2<i32>(-1,  1)));
        c = blend(c, sample_neighbor_if_ge(idx, pos, vec2<i32>( 1, -1)));
        c = blend(c, sample_neighbor_if_ge(idx, pos, vec2<i32>( 1,  1)));

        // Now all the orthogonal ones
        c = blend(c, sample_neighbor_if_ge(idx, pos, vec2<i32>(-1,  0)));
        c = blend(c, sample_neighbor_if_ge(idx, pos, vec2<i32>( 1,  0)));
        c = blend(c, sample_neighbor_if_ge(idx, pos, vec2<i32>( 0, -1)));
        c = blend(c, sample_neighbor_if_ge(idx, pos, vec2<i32>( 0,  1)));

        idx++;
    }

    return c;
}

fn render_perspective_underhangs(color: vec4<f32>, pos: MapPosition) -> vec4<f32> {
    var c = color;
    if (map.perspective_overhang_mask & 0x03u) == 0x03u { c = blend(c, sample_neighbor(pos, vec2<i32>( -1, -1))); }
    if (map.perspective_overhang_mask & 0x06u) == 0x06u { c = blend(c, sample_neighbor(pos, vec2<i32>( -1,  1))); }
    if (map.perspective_overhang_mask & 0x09u) == 0x09u { c = blend(c, sample_neighbor(pos, vec2<i32>(  1, -1))); }
    if (map.perspective_overhang_mask & 0x0cu) == 0x0cu { c = blend(c, sample_neighbor(pos, vec2<i32>(  1,  1))); }

    if (map.perspective_overhang_mask & 0x01u) == 0x01u { c = blend(c, sample_neighbor(pos, vec2<i32>(  0, -1))); }
    if (map.perspective_overhang_mask & 0x02u) == 0x02u { c = blend(c, sample_neighbor(pos, vec2<i32>( -1,  0))); }
    if (map.perspective_overhang_mask & 0x04u) == 0x04u { c = blend(c, sample_neighbor(pos, vec2<i32>(  0,  1))); }
    if (map.perspective_overhang_mask & 0x08u) == 0x08u { c = blend(c, sample_neighbor(pos, vec2<i32>(  1,  0))); }
    return c;
}

fn render_perspective_overhangs(color: vec4<f32>, pos: MapPosition) -> vec4<f32> {
    var c = color;
    if (map.perspective_overhang_mask & 0x01u) == 0x01u { c = blend(c, sample_neighbor(pos, vec2<i32>(  0,  1))); }
    if (map.perspective_overhang_mask & 0x02u) == 0x02u { c = blend(c, sample_neighbor(pos, vec2<i32>(  1,  0))); }
    if (map.perspective_overhang_mask & 0x04u) == 0x04u { c = blend(c, sample_neighbor(pos, vec2<i32>(  0, -1))); }
    if (map.perspective_overhang_mask & 0x08u) == 0x08u { c = blend(c, sample_neighbor(pos, vec2<i32>( -1,  0))); }

    if (map.perspective_overhang_mask & 0x03u) == 0x03u { c = blend(c, sample_neighbor(pos, vec2<i32>(  1,  1))); }
    if (map.perspective_overhang_mask & 0x06u) == 0x06u { c = blend(c, sample_neighbor(pos, vec2<i32>(  1, -1))); }
    if (map.perspective_overhang_mask & 0x09u) == 0x09u { c = blend(c, sample_neighbor(pos, vec2<i32>( -1,  1))); }
    if (map.perspective_overhang_mask & 0x0cu) == 0x0cu { c = blend(c, sample_neighbor(pos, vec2<i32>( -1, -1))); }
    return c;
}


@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    // XXX TODO DEBUG
    // DEBUG: Render atlas texture for testing
    //return textureSample(
    //    atlas_texture, atlas_sampler, in.uv
    //);
    // DEBUG: Render a single tile TODO XXX
    /*
    var tile_start = vec2<f32>(0.0, 0.0);
    var tile_offset = in.uv * 64.0;
    var rect_offset = tile_offset + map.tile_anchor_point * map.tile_size;
    var total_offset = tile_start + rect_offset;
    return textureSample(
        atlas_texture, atlas_sampler, total_offset / map.atlas_size
    );
    */


    var world_position = in.world_position.xy;

    // XXX TODO DEBUG Pink background for debugging
    var color = vec4<f32>(1.0, 0.0, 1.0, 1.0);

    var pos = world_to_tile_and_offset(world_position);
    var index = get_tile_index(pos.tile);

    if map.overhang_mode == 1u {
        color = render_perspective_underhangs(color, pos);
    }

    color = blend(color, sample_tile(map, index, pos.offset));

    if map.overhang_mode == 0u {
        color = render_dominance_overhangs(color, index, pos);
    }

    if map.overhang_mode == 1u {
        color = render_perspective_overhangs(color, pos);
    }

    return color;
}
