//! Example illustrating alternative map initialization and live updates.
//! We're maintaining a 1024x1024 map here that is changed every frame, so
//! thats megabytes of updates every frame we need to refresh on the CPU.
//! For this reason, this example will run with much lower FPS on most machines than the other
//! ones.
//!
//! Compiling this with --release can make a huge difference (for me 30 vs 200 FPS)

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::{FastTileMapPlugin, Map, MapBundleManaged};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::Rng;

#[path = "common/mouse_controls_camera.rs"]
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
            //WorldInspectorPlugin::new(),
            MouseControlsCameraPlugin::default(),
            FastTileMapPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, change_map)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
) {
    commands.spawn(Camera2dBundle::default());

    let tiles_texture = asset_server.load("pixel_tiles_16.png");

    let map = Map::builder(
        // Map size
        uvec2(1024, 1024),
        // Tile atlas
        tiles_texture,
        // Tile Size
        vec2(16., 16.),
    )
    .build_and_set(|_| 2);

    commands.spawn(MapBundleManaged::new(map, materials.as_mut()));
}

/// Update random patches of tile indices in the map.
fn change_map(mut materials: ResMut<Assets<Map>>, maps: Query<&Handle<Map>>) {
    let mut rng = rand::thread_rng();

    for map_handle in maps.iter() {
        let map = materials.get_mut(map_handle).unwrap();
        let mut m = map.indexer_mut();

        let k = rng.gen_range(5..50);
        let x_min = rng.gen_range(0..m.size().x - k);
        let y_min = rng.gen_range(0..m.size().y - k);
        let i = rng.gen_range(1..12);

        for y in y_min..y_min + k {
            for x in x_min..x_min + k {
                m.set(x, y, i);
            }
        }
    }
} // fn change_map
