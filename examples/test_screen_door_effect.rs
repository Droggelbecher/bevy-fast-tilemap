//! Test for "screen door effect" i.e. horizontal and/or vertical lines appearing between tiles.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2, vec3},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

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
            WorldInspectorPlugin::new(),
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, highlight_hovered)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
) {
    commands.spawn(Camera2dBundle {
        // On my specific machine this setting leads to an undesired vertical black line
        // between y=245 and y=246.
        // This maybe different for your environment.
        transform: Transform::from_translation(Vec3::new(-1480.93, 3046.95, 0.0))
            .with_scale(vec3(3.129, 3.129, 1.0)),
        ..default()
    });

    let map = Map::builder(
        uvec2(640, 640),
        // Due to its odd size, this atlas may lead to occasional "screen door"
        // effect, (horizontal or vertical lines) depending on the zoom level.
        // asset_server.load("debug_32x32_pad_1x1.png"),

        // Padding the atlas to a power of two sizes solves this for most occasions.
        // This may make it not contain a whole number of tiles so you should
        // use .with_n_tiles(...) below to specify the number of tiles you want to acutally use.
        asset_server.load("debug_32x32_pad_1x1_resized.png"),
        vec2(32., 32.),
    )
    // This is necessary for the "resized" (padded) atlas to work
    .with_n_tiles(Some(uvec2(8, 6)))
    // our atlas has a black padding of 1 px around each tile.
    // this will be very apparent if tiles don't line up perfectly.
    .with_padding(vec2(1.0, 1.0), vec2(1.0, 1.0), vec2(1.0, 1.0))
    .build_and_initialize(|m| {
        // Initialize using a closure
        for y in 0..m.size().y {
            for x in 0..m.size().x {
                m.set(x, y, 29);
            }
        }
    });

    commands.spawn(MapBundleManaged {
        material: materials.add(map),
        ..default()
    });
}

/// Highlight the currently hovered tile red, reset all other tiles
fn highlight_hovered(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut camera_query: Query<(&GlobalTransform, &Camera), With<OrthographicProjection>>,
    maps: Query<&Handle<Map>>,
    materials: Res<Assets<Map>>,
) {
    for event in cursor_moved_events.read() {
        for map_handle in maps.iter() {
            let map = materials.get(map_handle).unwrap();

            for (global, camera) in camera_query.iter_mut() {
                // Translate viewport coordinates to world coordinates
                if let Some(world) = camera
                    .viewport_to_world(global, event.position)
                    .map(|ray| ray.origin.truncate())
                {
                    // The map can convert between world coordinates and map coordinates for us
                    let coord = map.world_to_map(world);
                    println!("Map coordinate: {:?}", coord);
                } // if Some(world)
            } // for (global, camera)
        } // for map
    } // for event
} // highlight_hovered
