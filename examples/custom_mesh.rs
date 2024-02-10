//! Simple example illustrating use of a custom, non-rectangular mesh.
//! This is based on `layers`, so you might want to read that one first.
//! Each map is a single quad so the performance overhead should be low for a reasonable amount of
//! layers.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2, vec3},
    prelude::*,
    sprite::Mesh2dHandle,
    window::PresentMode,
};
use bevy_fast_tilemap::{FastTileMapPlugin, Map, MapBundle, MapIndexer, MeshManagedByMap};

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
            MouseControlsCameraPlugin::default(),
            FastTileMapPlugin::default(),
        ))
        .add_systems(Startup, startup)
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
    mut materials: ResMut<Assets<Map>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map = Map::builder(
        // Map size
        uvec2(51, 51),
        // Tile atlas
        asset_server.load("pixel_tiles_16.png"),
        // Tile Size
        vec2(16., 16.),
    )
    .build_and_initialize(|m| {
        // Initialize using a closure
        // Set all tiles in layer 0 to index 4
        for y in 0..m.size().y {
            for x in 0..m.size().y {
                m.set(x, y, ((x + y) % 4 + 1) as u32);
            }
        }
    });

    let mesh = Mesh2dHandle(meshes.add(Mesh::from(shape::Circle::new(300.0))));

    commands
        .spawn(MapBundle::new(map, materials.as_mut()))
        .insert(MapLayer(0))
        // Remove the `MeshManagedByMap` marker component so the map doesn't try to resize it
        .remove::<MeshManagedByMap>()
        // Insert our custom mesh
        .insert(mesh.clone());

    let map = Map::builder(
        uvec2(51, 51),
        asset_server.load("pixel_tiles_16.png"),
        vec2(16., 16.),
    )
    .build_and_initialize(initialize_layer1);

    let mut bundle = MapBundle::new(map, materials.as_mut());
    // Higher z value means "closer to the camera"
    bundle.transform = Transform::default().with_translation(vec3(0., 0., 1.));

    commands.spawn(bundle).insert(MapLayer(1)).insert(mesh);
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
            m.set(x, y, 11u32);
            //m.set(x, y, (x % 12) as u16);
        } // for x
    } // for y
}
