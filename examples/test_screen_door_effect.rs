//! Test for "screen door effect" i.e. horizontal or vertical lines appearing between tiles.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::prelude::*;

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;
use rand::Rng as _;

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
            FastTileMapPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
) {
    commands.spawn(Camera2dBundle::default());

    let rng = &mut rand::thread_rng();

    let map = Map::builder(
        uvec2(64, 64),
        asset_server.load("debug_32x32_pad_1x1.png"),
        vec2(32., 32.),
    )
    .with_padding(vec2(1.0, 1.0), vec2(1.0, 1.0), vec2(1.0, 1.0))
    .build_and_initialize(|m| {
        // Initialize using a closure
        // Set all tiles in layer 0 to index 4
        for y in 0..m.size().y {
            for x in 0..m.size().x {
                m.set(x, y, rng.gen_range(0..6*8) as u32);
            }
        }
    });

    commands.spawn(MapBundleManaged {
        material: materials.add(map),
        ..default()
    });

}
