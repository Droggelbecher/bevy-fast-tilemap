use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{ivec2, vec2},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::{
    bundle::FastTileMapDescriptor,
    map::Map,
    plugin::FastTileMapPlugin,
};
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
        .add_system(change_map)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    let tiles_texture = asset_server.load("simple_tiles_64.png");

    FastTileMapDescriptor {
        map_size: ivec2(1024, 1024),
        tile_size: vec2(64., 64.),
        tiles_texture,
        ..default()
    }
    .spawn(&mut commands, &mut images, &mut meshes);
}

/// Update random patches of tile indices in the map
fn change_map(
    mut images: ResMut<Assets<Image>>,
    mut maps: Query<&mut Map>,
) {
    let mut rng = rand::thread_rng();

    for mut map in maps.iter_mut() {
        let mut m = match map.get_mut(&mut *images) {
            Err(e) => {
                warn!("no map: {:?}", e);
                continue
            }
            Ok(x) => x,
        };

        let k = rng.gen_range(5..50);
        let x_min = rng.gen_range(0..m.size().x - k);
        let y_min = rng.gen_range(0..m.size().y - k);
        let i = rng.gen_range(1..8);

        for y in y_min .. y_min + k {
            for x in x_min .. x_min + k {
                m[ivec2(x, y)] = i;
            }
        }
    }
} // fn change_map
