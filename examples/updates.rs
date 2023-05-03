//! Example illustrating alternative map initialization and live updates (here: every frame).
//!

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::{FastTileMapPlugin, Map, MapBundle, MapReadyEvent, MeshManagedByMap};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::Rng;

mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Fast Tilemap example"),
                resolution: (1820., 920.).into(),
                // disable vsync so we can see the raw FPS speed
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(FastTileMapPlugin::default())
        .add_plugin(MouseControlsCameraPlugin::default())
        .add_startup_system(startup)
        .add_system(initialize_map)
        .add_system(change_map)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
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
    .build(&mut images);

    commands.spawn(MapBundle::new(map))
        .insert(MeshManagedByMap);
}

/// Check whether the map is ready to be filled with contents and do so.
/// This way the map gets initialized as soon as its texture is available in the asset server.
/// See the other examples for the slightly more convenient immediate initialization.
fn initialize_map(
    mut evs: EventReader<MapReadyEvent>,
    mut images: ResMut<Assets<Image>>,
    mut maps: Query<&mut Map>,
) {
    // Once the map texture is loaded we'll receive a `MapReadyEvent`.
    // When this happens is a good point in time to initialize our map contents
    for ev in evs.iter() {
        // Get the actual map. Since it sent us an event,
        // this should not fail.
        let map = maps.get_mut(ev.map).unwrap();

        // Get the indexer for modifying the map texture.
        // Since we got the MapReadyEvent, this should be available in `images`,
        // so this should also not fail.
        if let Ok(mut m) = map.get_mut(&mut *images) {
            for y in 0..m.size().y {
                for x in 0..m.size().x {
                    m.set(x, y, 1u16);
                }
            }
        }
    } // for ev
} // generate_map

/// Update random patches of tile indices in the map
fn change_map(mut images: ResMut<Assets<Image>>, maps: Query<&Map>) {
    let mut rng = rand::thread_rng();

    for map in maps.iter() {
        // Get the indexer into the map texture
        let mut m = match map.get_mut(&mut *images) {
            Err(e) => {
                // Map texture is not available
                warn!("no map: {:?}", e);
                continue;
            }
            Ok(x) => x,
        };

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
