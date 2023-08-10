use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::{uvec2, vec2, vec3};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_fast_tilemap::{FastTileMapPlugin, Map, MapBundle, MapIndexer, MeshManagedByMap};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

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
            WorldInspectorPlugin::new(),
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    let k = 10;

    for i in 0..5 {
        for j in 0..5 {
            let map = Map::builder(
                // Map size
                uvec2(k, k),
                // Tile atlas
                asset_server.load("pixel_tiles_16.png"),
                // Tile Size
                vec2(16., 16.),
            )
            .build_and_set(&mut images, |pos| {
                // Initialize using a closure
                // Set all tiles in layer 0 to index 4
                ((i + j) % 4 + 6) as u16
            });

            commands
                .spawn(MapBundle::new(map))
                // Have the map manage our mesh so it always has the right size
                .insert((
                    MeshManagedByMap,
                    Transform {
                        translation: Vec3::new((i * k) as f32 * 16.0, (j * k) as f32 * 16.0, 1.0),
                        ..default()
                    },
                ));
        }
    }
}
