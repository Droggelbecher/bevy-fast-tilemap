use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{ivec2, vec2},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::{
    bundle::FastTileMapDescriptor, map::MapLayerMaterial, plugin::FastTileMapPlugin,
};

mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<MapLayerMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Create map with (10 * 128) ^ 2 tiles or 1,638,400 tiles.
    FastTileMapDescriptor {
        map_size: ivec2(1280, 1280),
        tiles_textures: vec![asset_server.load("tiles.png")],
        tile_size: vec2(16., 16.),
    }
    .spawn(&mut commands, &mut images, &mut meshes, &mut materials);
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Benchmark Example"),
            present_mode: PresentMode::Immediate,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(FastTileMapPlugin::default())
        .add_plugin(MouseControlsCameraPlugin::default())
        .add_startup_system(startup)
        .run();
}
