//! Simple example for illustrating axonometrically projected tilemaps.
//! To keep the math simple instead of strictly isometric, we stick to a projection
//! where each tile ends up a diamond shape that is twice as wide as high.

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::{uvec2, vec2};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_fast_tilemap::{
    FastTileMapPlugin, Map, MapBundle, MapIndexer, MeshManagedByMap, IDENTITY,
};
use rand::Rng;

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
        .add_systems(Update, show_coordinate)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map = Map::builder(
        // Map size
        uvec2(100, 100),
        // Tile atlas
        asset_server.load("dominance.png"),
        // Tile size
        vec2(118.0, 118.0),
    )
    .with_projection(IDENTITY)
    .with_padding(
        vec2(118.0, 118.0),
        // top/left padding
        vec2(118., 118.),
        // bottom/right padding
        vec2(118., 118.),
    )
    // "Dominance" overhang draws the overlap of tiles depending on their index in the tile atlas.
    // Tiles with higher index will be drawn on top of tiles with lower index.
    // For this we draw in the "padding" area of the tile atlas.
    //
    // This requires each pixel to be computed once for every level higher than the current one
    // and for every neighbor which can be a drastic performance hit.
    // Therefore its a good idea to limit the number of levels looked upwards here.
    .with_dominance_overhang(3)
    .build_and_initialize(&mut images, init_map);

    commands.spawn(MapBundle::new(map)).insert(MeshManagedByMap);
} // startup

/// Fill the map with a random pattern
fn init_map(m: &mut MapIndexer) {
    let mut rng = rand::thread_rng();
    for y in 0..m.size().y {
        for x in 0..m.size().x {
            m.set(x, y, rng.gen_range(0..3));
        }
    }
} // reset_map

/// Highlight the currently hovered tile red, reset all other tiles
fn show_coordinate(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut camera_query: Query<(&GlobalTransform, &Camera), With<OrthographicProjection>>,
    maps: Query<&Map>,
) {
    for event in cursor_moved_events.iter() {
        for map in maps.iter() {
            for (global, camera) in camera_query.iter_mut() {
                // Translate viewport coordinates to world coordinates
                if let Some(world) = camera
                    .viewport_to_world(global, event.position)
                    .map(|ray| ray.origin.truncate())
                {
                    // The map can convert between world coordinates and map coordinates
                    let coord = map.world_to_map(world);
                    println!("Map coordinate: {:?}", coord);
                } // if Some(world)
            } // for (global, camera)
        } // for map
    } // for event
} // highlight_hovered
