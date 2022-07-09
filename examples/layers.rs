use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::{ivec2, vec2};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_fast_tilemap::{
    bundle::FastTileMapDescriptor,
    map::{Map, MapLayer, MapLayerMaterial, MapReadyEvent},
    plugin::FastTileMapPlugin,
};

mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1820.,
            height: 920.,
            // disable vsync so we can see the raw FPS speed
            present_mode: PresentMode::Immediate,
            title: String::from("Fast Tilemap example"),
            ..default()
        })
        .add_plugins(DefaultPlugins)
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
    mut materials: ResMut<Assets<MapLayerMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    FastTileMapDescriptor {
        map_size: ivec2(51, 51),
        tiles_textures: vec![
            asset_server.load("pixel_tiles_16.png"),
            asset_server.load("pixel_tiles_16.png"),
        ],
        tile_size: vec2(16., 16.),
    }
    .spawn(&mut commands, &mut images, &mut meshes, &mut materials);
}

fn generate_map(
    mut evs: EventReader<MapReadyEvent>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<MapLayerMaterial>>,
    mut maps: Query<&Map>,
    mut map_layers: Query<&mut MapLayer>,
) {
    for ev in evs.iter() {
        // map is ready so this should not fail
        let map = maps.get_mut(ev.map).unwrap();

        let mut m = match map.get_mut(&mut map_layers, &mut *materials, &mut *images) {
            Err(_) => continue,
            Ok(x) => x,
        };

        // Set all tiles in layer 0 to index 4

        for y in 0..map.size().y {
            for x in 0..map.size().x {
                m[0][ivec2(x, y)] = ((x + y) % 4 + 1) as u16;
            }
        }

        // Define some sub-rectangle in the center of the map

        let k = 10;

        let y_min = map.size().y / 2 - k;
        let x_min = map.size().x / 2 - k;
        let y_max = map.size().y / 2 + k + 1;
        let x_max = map.size().x / 2 + k + 1;

        for y in y_min..y_max {
            for x in x_min..x_max {

                // Set tile in layer 1 to index 11
                m[1][ivec2(x, y)] = 11;

                // Set RGBA tint of tile in layer 1.
                // RGBA values get multiplied, i.e. A channel can directly be used to control
                // transparency.
                // In this example x position influences the green channel.
                // Default tint is [1.0, 1.0, 1.0, 1.0].
                m[1].set_tint(
                    ivec2(x, y),
                    [
                        1.0,
                        1.0,
                        (x - x_min) as f32 / (x_max - x_min) as f32,
                        (y - y_min) as f32 / (y_max - y_min) as f32,
                    ],
                );
            }
        }
    } // for ev
}
