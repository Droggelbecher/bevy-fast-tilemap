//! Test for "screen door effect" i.e. horizontal or vertical lines appearing between tiles.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, math::{uvec2, vec2, vec3}, prelude::*, sprite::Mesh2dHandle, window::PresentMode
};
use bevy_fast_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;
use rand::Rng as _;

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
            WorldInspectorPlugin::new()
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, highlight_hovered)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
    // mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2dBundle {
        // transform: Transform::from_translation(Vec3::new(1098.5, 1011.5, 0.0)),
        transform: Transform::from_translation(Vec3::new(-1480.93, 3046.95, 0.0))
            .with_scale(vec3(3.129, 3.129, 1.0)),
        ..default()
    });

    let rng = &mut rand::thread_rng();

    let map = Map::builder(
        uvec2(640, 640),
        // This still gives occasional screen door effect, depending on the zoom level
        // asset_server.load("debug_32x32_pad_1x1.png"),

        // This seems to work now
        asset_server.load("debug_32x32_pad_1x1_resized.png"),
        vec2(32., 32.),
    )

    // This is necessary for the "resized" version to be usable
    .with_n_tiles(Some(uvec2(8, 6)))
    .with_padding(vec2(1.0, 1.0), vec2(1.0, 1.0), vec2(1.0, 1.0))
    .build_and_initialize(|m| {
        // Initialize using a closure
        // Set all tiles in layer 0 to index 4
        for y in 0..m.size().y {
            for x in 0..m.size().x {
                m.set(x, y, 29);
            }
        }
    });

    commands.spawn(MapBundleManaged {
        material: materials.add(map),
        ..default()
    })
    ;

}

/// Highlight the currently hovered tile red, reset all other tiles
fn highlight_hovered(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut camera_query: Query<(&GlobalTransform, &Camera), With<OrthographicProjection>>,
    maps: Query<&Handle<Map>>,

    // We'll actually change the map contents for highlighting
    mut materials: ResMut<Assets<Map>>,
) {
    for event in cursor_moved_events.read() {
        for map_handle in maps.iter() {
            let map = materials.get_mut(map_handle).unwrap();

            for (global, camera) in camera_query.iter_mut() {
                // Translate viewport coordinates to world coordinates
                if let Some(world) = camera
                    .viewport_to_world(global, event.position)
                    .map(|ray| ray.origin.truncate())
                {
                    // The map can convert between world coordinates and map coordinates for us
                    let coord = map.world_to_map(world);
                    println!("Map coordinate: {:?}", coord);

                    // let coord = coord
                    //     .as_uvec2()
                    //     .clamp(uvec2(0, 0), map.map_size() - uvec2(1, 1));

                    // Modifying the map requires that the underlying data be synchronized to
                    // the GPU again so you want to avoid to do this every frame if your map is
                    // very large. The transfer cost does not depend on how much you change, so
                    // you may as well generate the whole thing (of course consider the actual
                    // generation time).
                    //
                    // Note that this technically does *not* modify the `Map` component, but
                    // the underlying data which is stored in the material.
                    // let mut m = map.indexer_mut();

                    // reset_map(&mut m);
                    // m.set_uvec(coord, 3u32);
                } // if Some(world)
            } // for (global, camera)
        } // for map
    } // for event
} // highlight_hovered
