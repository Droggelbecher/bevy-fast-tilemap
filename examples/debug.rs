//! Simple example illustrating how to use updates to the tilemap for animation.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::{bundle::MapBundle, FastTileMapPlugin, Map};

mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

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
        // Performance-wise you can step this much faster but it'd require an epillepsy warning.
        .insert_resource(Time::<Fixed>::from_seconds(0.2))
        .run();
}

#[derive(Component)]
struct AnimationLayer;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map = Map::builder(
        uvec2(8, 8),
        asset_server.load("debug01.png"),
        vec2(64., 64.),
    )
    .build_and_initialize(|m| {
        // Initialize using a closure
        // Set all tiles in layer 0 to index 4
        for y in 0..m.size().y {
            for x in 0..m.size().y {
                m.set(x, y, (((x + y) * 19) % 64 + 1) as u32);
            }
        }
    });

    commands.spawn(MapBundle {
        material: materials.add(map),
        ..default()
    });
}
