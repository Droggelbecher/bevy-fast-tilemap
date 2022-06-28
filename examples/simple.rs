use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::{ivec2, vec2};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_fast_tilemap::{
    bundle::FastTileMapDescriptor,
    map::{Map, MapLayer, MapLayerMaterial},
    plugin::FastTileMapPlugin,
};
use rand::Rng;

mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

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
        .add_system(change_map)
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
        map_size: ivec2(1024, 1024),
        tiles_textures: vec![asset_server.load("simple_tiles_64.png")],
        tile_size: vec2(64., 64.),
    }
    .spawn(commands, &mut images, &mut meshes, &mut materials);
}

/// Update random patches of tile indices in the map
fn change_map(
    mut materials: ResMut<Assets<MapLayerMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut maps: Query<&Map>,
    mut map_layers: Query<&mut MapLayer>,
) {
    let mut rng = rand::thread_rng();

    for map in maps.iter_mut() {

        let mut m = match map.get_mut(&mut map_layers, &mut *materials, &mut *images) {
            Err(_) => continue,
            Ok(x) => x,
        };

        let k = rng.gen_range(5..50);
        let x_min = rng.gen_range(0..map.map_size.x - k);
        let y_min = rng.gen_range(0..map.map_size.y - k);
        let i = rng.gen_range(1..8);

        let tint = [
            rng.gen_range(0.5 .. 1.) as f32,
            rng.gen_range(0.5 .. 1.) as f32,
            rng.gen_range(0.5 .. 1.) as f32,
            1.0
        ];

        for y in y_min .. y_min + k {
            for x in x_min .. x_min + k {
                m[(0, ivec2(x, y))] = i;
                m.set_tint(0, ivec2(x, y), tint);
            }
        }
    }
} // fn change_map
