//! Simple example illustrating how to use updates to the tilemap for animation.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2, vec3},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::{FastTileMapPlugin, Map, MapBundle, MeshManagedByMap};

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
        .add_system(update_map.in_schedule(CoreSchedule::FixedUpdate))
        // Performance-wise you can step this much faster but it'd require an epillepsy warning.
        .insert_resource(FixedTime::new_from_secs(0.2))
        .run();
}

#[derive(Component)]
struct AnimationLayer;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map = Map::builder(
        uvec2(50, 50),
        asset_server.load("pixel_tiles_16.png"),
        vec2(16., 16.),
    )
    .build_and_initialize(&mut images, |m| {
        // Initialize using a closure
        // Set all tiles in layer 0 to index 4
        for y in 0..m.size().y {
            for x in 0..m.size().y {
                m.set(x, y, ((x + y) % 4 + 1) as u16);
            }
        }
    });

    commands.spawn(MapBundle::new(map)).insert(MeshManagedByMap);

    let map = Map::builder(
        uvec2(50, 50),
        asset_server.load("pixel_tiles_16.png"),
        vec2(16., 16.),
    )
    .build(&mut images);

    let mut bundle = MapBundle::new(map);
    bundle.transform = Transform::default().with_translation(vec3(0., 0., 1.));

    commands
        .spawn(bundle)
        .insert(MeshManagedByMap)
        .insert(AnimationLayer);
}

fn update_map(mut images: ResMut<Assets<Image>>, maps: Query<&Map, With<AnimationLayer>>) {
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

        let k = 10;
        let y_min = m.size().y / 2 - k;
        let x_min = m.size().x / 2 - k;
        let y_max = m.size().y / 2 + k + 1;
        let x_max = m.size().x / 2 + k + 1;

        for y in y_min..y_max {
            for x in x_min..x_max {
                // Tile index transitions, since our animation is ridiculously short,
                // we can list them here explicitly
                let t = match m.at(x, y) {
                    6 => 7,
                    7 => 8,
                    _ => 6,
                };
                m.set(x, y, t);
            }
        }
    }
}
