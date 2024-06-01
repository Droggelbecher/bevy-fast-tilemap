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
                        return 2.0 * sin(x / pi);
                    }

                    fn sample_tile(in: ExtractIn) -> vec4<f32> {
                        var tile_index = in.tile_index;
                        var tile_position = in.tile_position;

                        // right boundary
                        var r = map.tile_size.x + offs(f32(in.tile_position.y) * map.tile_size.y + in.tile_offset.y);
                        // left boundary
                        var l = offs(f32(in.tile_position.y) * map.tile_size.y + in.tile_offset.y);
                        // top boundary
                        var t = map.tile_size.y + offs(f32(in.tile_position.x) * map.tile_size.x + in.tile_offset.x);
                        // bottom boundary
                        var b = offs(f32(in.tile_position.x) * map.tile_size.x + in.tile_offset.x);

                        var bw = 1.0;

                        if in.tile_offset.x > r - bw {
                            tile_position = tile_position + vec2<i32>(1, 0);
                        }
                        else if in.tile_offset.x <= l + bw {
                            tile_position = tile_position + vec2<i32>(-1, 0);
                        }

                        if in.tile_offset.y > t - bw {
                            tile_position = tile_position + vec2<i32>(0, 1);
                        }
                        else if in.tile_offset.y <= b + bw {
                            tile_position = tile_position + vec2<i32>(0, -1);
                        }

                        tile_index = get_tile_index(tile_position);

                        if (abs(in.tile_offset.x - r) <= bw || abs(in.tile_offset.x - l) <= bw
                            || abs(in.tile_offset.y - t) <= bw || abs(in.tile_offset.y - b) <= bw)

                            // TODO: this condition is too harsh, it should compare the tile index
                            // with the neighbor independently of whether we would end up sampling
                            // from it
                            && (tile_index != in.tile_index)
                        {
                            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
                        }

                        var color = sample_tile_at(tile_index, in.tile_position, in.tile_offset);
                        return color;
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
