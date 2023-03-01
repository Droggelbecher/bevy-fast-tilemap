struct Mesh {
	model: mat4x4<f32>;
};

struct View {
	view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> view: View;

[[group(2), binding(0)]]
var<uniform> mesh: Mesh;

[[group(1), binding(0)]]
var map_texture: texture_storage_2d<r16uint,read>;

[[group(1), binding(1)]]
var tint_texture: texture_storage_2d<rgba8uint,read>;

[[group(1), binding(2)]]
var tiles_texture: texture_2d<f32>;

[[group(1), binding(3)]]
var tiles_sampler: sampler;

struct TileMapMaterial {
	tilemap_size: vec2<f32>;
};

[[group(1), binding(4)]]
var<uniform> tilemap_material: TileMapMaterial;


struct Vertex {
	[[location(0)]] position: vec2<f32>;
	[[location(1)]] normal: vec2<f32>;
	[[location(2)]] uv: vec2<f32>;
};

struct VertexOutput {
	[[builtin(position)]] clip_position: vec4<f32>;
	[[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn vertex(v: Vertex) -> VertexOutput {
	var out: VertexOutput;
	let pos = view.view_proj * mesh.model * vec4<f32>(v.position, 0.0, 1.0);
	out.clip_position = pos;
	out.uv = v.uv;
	return out;
}

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {

	var map_size = textureDimensions(map_texture);
	var n_tiles = tilemap_material.tilemap_size.x * tilemap_material.tilemap_size.y; // TODO: Maybe Move this mult to CPU side

	// TODO: lots of int <-> float conversions here, can we reduce these?

	// Map position with fractional part
	var map_pos = in.uv * vec2<f32>(map_size);
	// Integer part of map position (tile coordinate)
	var map_coord = vec2<i32>(floor(map_pos));
	// fractional part (position inside tile)
	var offset = fract(map_pos);

	// tilemap index for that tile map_coord
	var index = f32(textureLoad(map_texture, map_coord).r);

	// Convert index to x/y tile position in tilemap
	var index_y = floor(index / tilemap_material.tilemap_size.y);
	var index_x = index - index_y * tilemap_material.tilemap_size.y;

	var tint_color = vec4<f32>(textureLoad(tint_texture, map_coord).rgba) / 255.;

	return textureSample(
		tiles_texture, tiles_sampler,
		(vec2<f32>(index_x, index_y) + offset) / tilemap_material.tilemap_size
	) * tint_color;
}
