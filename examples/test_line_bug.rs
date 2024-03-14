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
        uvec2(2048, 2048),
        // Tile atlas
        //asset_server.load("debug03_80.png"),
        asset_server.load("debug01.png"),
        // Tile size (pixels)
        vec2(64., 64.),
        //vec2(80., 80.),
    )
    .build_and_initialize(|m| {
        for x in 0..m.size().x {
            for y in 0..m.size().y {
                m.set(x, y, 11);
            }
        }
    });

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
