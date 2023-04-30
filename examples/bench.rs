
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
use bevy_fast_tilemap::{
    bundle::FastTileMapDescriptor, plugin::FastTileMapPlugin,
};

mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Create map with (10 * 128) ^ 2 tiles or 1,638,400 tiles.
    let bundle = FastTileMapDescriptor {
        map_size: uvec2(1280, 1280),
        tile_size: vec2(16., 16.),
        tiles_texture: asset_server.load("tiles.png"),
        ..default()
    }
    .build(&mut images, &mut meshes);

    commands.spawn(bundle);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Benchmark Example"),
                resolution: (1270.0, 720.0).into(),
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(FastTileMapPlugin::default())
        .add_plugin(MouseControlsCameraPlugin::default())
        .add_startup_system(startup)
        .run();
}
