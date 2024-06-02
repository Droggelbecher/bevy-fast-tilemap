use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    window::PresentMode,
};
use bevy_fast_tilemap::{CustomFastTileMapPlugin, FastTileMapPlugin, Map, MapBundleManaged};
use rand::Rng;

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

#[derive(Debug, Clone, Default, Reflect, AsBindGroup, ShaderType)]
struct UserData {
    x: u32,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Fast Tilemap example"),
                    resolution: (1820., 920.).into(),
                    // disable vsync so we can see the raw FPS speed
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            MouseControlsCameraPlugin::default(),
            CustomFastTileMapPlugin::<UserData> {
                user_code: Some(
                    r#"
                    struct UserData {
                        x: u32,
                    };

                    fn offs(x: f32) -> f32 {
                        // make sure the frequency is a multiple of the tile size
                        // so the corners of the tiles align
                        var pi = radians(180.0);
                        return 1.0 * sin(x * pi / 8.0);
                    }

                    fn sample_tile(in: ExtractIn) -> vec4<f32> {
                        // DEBUG
                        //if in.tile_offset.x <= 1.0 || in.tile_offset.y <= 1.0 {
                            //return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                        //}

                        var tile_index = in.tile_index;

                        // offs() is shifted this much inwards
                        var doffs = 2.0;

                        // right boundary
                        var r = map.tile_size.x + offs(f32(in.tile_position.y) * map.tile_size.y + in.tile_offset.y) - doffs;
                        // left boundary
                        var l = offs(f32(in.tile_position.y) * map.tile_size.y + in.tile_offset.y) + doffs;
                        // top boundary
                        var t = map.tile_size.y + offs(f32(in.tile_position.x) * map.tile_size.x + in.tile_offset.x) - doffs;
                        // bottom boundary
                        var b = offs(f32(in.tile_position.x) * map.tile_size.x + in.tile_offset.x) + doffs;

                        // border width
                        var bw = 0.2;

                        // are we in a neighbor tile (or in a border to it)?
                        var tile_offset = vec2<i32>(0, 0);

                        // determine actual tile index to render at this position
                        if in.tile_offset.x > r - bw {
                            tile_offset = tile_offset + vec2<i32>(1, 0);
                        }
                        else if in.tile_offset.x <= l + bw {
                            tile_offset = tile_offset + vec2<i32>(-1, 0);
                        }
                        if in.tile_offset.y > t - bw {
                            tile_offset = tile_offset + vec2<i32>(0, 1);
                        }
                        else if in.tile_offset.y <= b + bw {
                            tile_offset = tile_offset + vec2<i32>(0, -1);
                        }
                        // dominance rule: highest tile index wins
                        var tile_position = in.tile_position + tile_offset;
                        tile_index = get_tile_index(tile_position);
                        var max_index = max(in.tile_index, tile_index);

                        // Figure out border drawing.
                        // Borders are drawn only between tiles of different indices.
                        // In the easy case the border is between "this" tile index (in.tile_index)
                        // and some horizontal/vertical neighbor.
                        //
                        // It might however also be between a horizontal/vertical neighbor and a
                        // diagonal neighbor. In that case we need to compare our neighbors indices
                        // with each other

                        var is_border = false;

                        var is_xborder = abs(in.tile_offset.x - r) <= bw || abs(in.tile_offset.x - l) <= bw;
                        var is_yborder = abs(in.tile_offset.y - t) <= bw || abs(in.tile_offset.y - b) <= bw;

                        // diagonal case
                        if tile_offset.x != 0 && tile_offset.y != 0 {

                            //    ##===+===+===##===========##
                            //    ||   |   |   ||           ||
                            //    ||---+---+---||           ||
                            //    ||   | C |   ||     R     ||
                            //    ||---+---+---||           ||
                            //    ||   |   | d ||           ||
                            //    ##===+===+===##===========##
                            //    ||           ||           ||
                            //    ||           ||           ||
                            //    ||     B     ||     D     ||
                            //    ||           ||           ||
                            //    ||           ||           ||
                            //    ##===========##===========##
                            //

                            // The dominating tile index in a diagonal sector (d) is the max index of
                            // the current tile (C) and the three neighbors (R, B, D).
                            // `max_index` at this point is already max(C, D), so we only need to
                            // include R & B.
                            max_index = max(max_index, get_tile_index(in.tile_position + vec2<i32>(tile_offset.x, 0)));
                            max_index = max(max_index, get_tile_index(in.tile_position + vec2<i32>(0, tile_offset.y)));


                            if is_xborder && max(in.tile_index, get_tile_index(in.tile_position + vec2<i32>(0, tile_offset.y))) != max_index {
                                is_border = true;
                                //return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                            }
                            if is_yborder && max(in.tile_index, get_tile_index(in.tile_position + vec2<i32>(tile_offset.x, 0))) != max_index {
                                is_border = true;
                                //return vec4<f32>(1.0, 0.0, 1.0, 1.0);
                            }
                        }
                        else if (is_xborder || is_yborder) && max_index > in.tile_index {
                                is_border = true;
                                //return vec4<f32>(0.0, 1.0, 0.0, 1.0);
                        }

                        if is_border
                        {
                            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
                        }

                        var color = sample_tile_at(max_index, in.tile_position, in.tile_offset);
                        return color;
                    }

                    fn dominates_index(p: vec2<i32>, index: u32) -> bool {
                        return get_tile_index(p) > index;
                    }

                    "#.to_string()
                ),
                ..default()
            },
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map<UserData>>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map = Map::<UserData>::builder(
        uvec2(64, 64),
        asset_server.load("patterns.png"),
        vec2(16., 16.),
    )
    // A tile (pattern) in the atlas is 8x8 map tiles large
    // (i.e. it will repeat every 8 tiles in any direction)
    .with_atlas_tile_size_factor(8)
    .build_and_initialize(|m| {
        let mut rng = rand::thread_rng();

        for y in 0..m.size().y {
            for x in 0..m.size().y {
                m.set(x, y, rng.gen_range(0..2));
            }
        }
    });

    commands.spawn(MapBundleManaged {
        material: materials.add(map),
        ..default()
    });
}
