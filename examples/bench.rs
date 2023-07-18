//! Analog to https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/bench.rs
//! Note that we offer much less in terms of features compared to bevy_ecs_tilemap,
//! so the comparison might rightfully be considered unfair.
//! In terms of raw speed this should be quite a bit faster though at time of this writing.
//! Also, we set PresentMode::Immediate so we can measure a FPS above the VSync rate.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::{FastTileMapPlugin, Map, MapBundle, MeshManagedByMap};

mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Create map with (10 * 128) ^ 2 tiles or 1,638,400 tiles.
    let map = Map::builder(
        // Map size (tiles)
        uvec2(1280, 1280),
        // Tile atlas
        asset_server.load("tiles.png"),
        // Tile size (pixels)
        vec2(16., 16.),
    )
    .build(&mut images);

    commands
        .spawn(MapBundle::new(map))
        // Have the map manage our mesh so it always has the right size
        .insert(MeshManagedByMap);
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
