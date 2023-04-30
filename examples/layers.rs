
//! Simple example illustrating how to use multiple Map instances as layers.
//! Each map is a single quad so the performance overhead should be low for a reasonable amount of
//! layers.

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::{uvec2, vec2, vec3};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_fast_tilemap::{
    MapDescriptor, MapIndexer, FastTileMapPlugin,
};

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
        .add_plugin(MouseControlsCameraPlugin::default())
        .add_plugin(FastTileMapPlugin::default())
        .add_startup_system(startup)
        .run();
}

// Completely optional:
// Add an extra component to keep track of which map layer is which for easy modification later,
// potentially containing some additional information.
// Since you likely want the layer to have different z-coordinates you could also use that to
// distinguish them.

#[derive(Component)]
struct MapLayer(i32);

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    let bundle = MapDescriptor {
        map_size: uvec2(51, 51),
        tile_size: vec2(16., 16.),
        tiles_texture: asset_server.load("pixel_tiles_16.png"),
        ..default()
    }
    .build_and_initialize(&mut images, &mut meshes, |m| {
        // Initialize using a closure
        // Set all tiles in layer 0 to index 4
        for y in 0..m.size().y {
            for x in 0..m.size().x {
                m.set(x, y, ((x + y) % 4 + 1) as u16);
            }
        }
    });

    commands.spawn(bundle).insert(MapLayer(0));

    let bundle = MapDescriptor {
        map_size: uvec2(51, 51),
        tile_size: vec2(16., 16.),
        tiles_texture: asset_server.load("pixel_tiles_16.png"),
        // Higher z value means "closer to the camera"
        transform: Transform::default().with_translation(vec3(0., 0., 1.)),
        ..default()
    }
    // Initialize using a function
    .build_and_initialize(&mut images, &mut meshes, initialize_layer1);

    commands.spawn(bundle).insert(MapLayer(1));
}

fn initialize_layer1(m: &mut MapIndexer) {
    // Define some sub-rectangle in the center of the map
    // and set tile to index 11 for all of these

    let k = 10;
    let y_min = m.size().y / 2 - k;
    let x_min = m.size().x / 2 - k;
    let y_max = m.size().y / 2 + k + 1;
    let x_max = m.size().x / 2 + k + 1;

    for y in y_min..y_max {
        for x in x_min..x_max {
            m.set(x, y, 11);
        } // for x
    } // for y
}
