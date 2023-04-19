use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::{ivec2, vec2, vec3};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_fast_tilemap::{
    bundle::FastTileMapDescriptor,
    map::{Map, MapReadyEvent},
    plugin::FastTileMapPlugin,
};

mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Fast Tilemap example"),
                    resolution: (1820., 920.).into(),
                    // disable vsync so we can see the raw FPS speed
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            })
        )
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(MouseControlsCameraPlugin::default())
        .add_plugin(FastTileMapPlugin::default())
        .add_startup_system(startup)
        .add_system(generate_map)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    FastTileMapDescriptor {
        map_size: ivec2(51, 51),
        tile_size: vec2(16., 16.),
        tiles_texture: asset_server.load("pixel_tiles_16.png"),
        transform: default(),
    }
    .spawn(&mut commands, &mut images, &mut meshes);

    FastTileMapDescriptor {
        map_size: ivec2(51, 51),
        tile_size: vec2(16., 16.),
        tiles_texture: asset_server.load("pixel_tiles_16.png"),
        transform: Transform::default().with_translation(vec3(0., 0., 1.)),
    }
    .spawn(&mut commands, &mut images, &mut meshes);
}

fn generate_map(
    mut evs: EventReader<MapReadyEvent>,
    mut images: ResMut<Assets<Image>>,
    mut maps: Query<(&mut Map, &Transform)>,
) {
    for ev in evs.iter() {
        // map is ready so this should not fail
        let (mut map, transform) = maps.get_mut(ev.map).unwrap();

        let mut m = match map.get_mut(&mut *images) {
            Err(_) => continue,
            Ok(x) => x,
        };

        // For simplicity, we identify layer here by z coordinate.
        // As this is a float comparison this is not generally advisable,
        // in real code you may prefer to add an extra component to identify the layer
        if transform.translation.z == 0.0 {
            // Set all tiles in layer 0 to index 4
            for y in 0..map.size().y {
                for x in 0..map.size().x {
                    m[ivec2(x, y)] = ((x + y) % 4 + 1) as u16;
                }
            }
        } else {
            // Define some sub-rectangle in the center of the map
            // and set tile to index 11 for all of these

            let k = 10;
            let y_min = map.size().y / 2 - k;
            let x_min = map.size().x / 2 - k;
            let y_max = map.size().y / 2 + k + 1;
            let x_max = map.size().x / 2 + k + 1;

            for y in y_min..y_max {
                for x in x_min..x_max {
                    m[ivec2(x, y)] = 11;
                } // for x
            } // for y
        }
    } // for ev
}
