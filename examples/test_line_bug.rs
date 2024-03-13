//! Analog to https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/bench.rs
//! Note that we offer much less in terms of features compared to bevy_ecs_tilemap,
//! so the comparison might rightfully be considered unfair.
//! In terms of raw speed this should be quite a bit faster though at time of this writing.
//! Also, we set PresentMode::Immediate so we can measure a FPS above the VSync rate.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2, vec3},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::{FastTileMapPlugin, Map, MapBundleManaged};

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;
use rand::Rng;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Create map with (10 * 128) ^ 2 tiles or 1,638,400 tiles.
    let map = Map::builder(
        // Map size (tiles)
        uvec2(1280, 1280),
        // Tile atlas
        asset_server.load("debug03_80.png"),
        // Tile size (pixels)
        vec2(80., 80.),
    )
    .build_and_initialize(|m| {
        for x in 0..1280 {
            for y in 0..1280 {
                m.set(x, y, 4);
            }
        }
    });

    let mut max_err = (Vec2::ZERO, Vec2::ZERO, 0.0);

    for _ in 0..100000 {
        // Set pos to a random 2d position
        let rng = &mut rand::thread_rng();
        let pos = vec2(
            rng.gen_range(-64000.0..64000.0),
            rng.gen_range(-64000.0..64000.0),
        );
        let pos2 = map.map_to_world_3d(map.world_to_map(pos).extend(0.0));
        let err = (pos - pos2.xy()).length();
        if err > max_err.2 {
            max_err = (pos, pos2.xy(), err);
        }
    }
    error!("max_err: {:?}", max_err);

    let mut bundle = MapBundleManaged::new(map, materials.as_mut());

    bundle.transform = Transform::default().with_translation(vec3(0., 0., -100.0));

    commands.spawn(bundle);
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Benchmark Example"),
                    resolution: (1270.0, 720.0).into(),
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            MouseControlsCameraPlugin::default(),
            FastTileMapPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .run();
}
