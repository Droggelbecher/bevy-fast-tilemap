//! This example also demonstrates how to insert custom code into the tilemap shader to
//! modify the appearance of the tiles. In this case, we add a "special" bit to the tile index
//! and use it to make some tiles bounce up and down and tint them red.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    window::PresentMode,
};

use bevy_fast_tilemap::{
    bundle::MapBundleManaged, map::MapIndexer, CustomFastTileMapPlugin, Map, AXONOMETRIC,
};
use rand::Rng;

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

#[derive(Debug, Clone, Default, Reflect, AsBindGroup, ShaderType)]
struct UserData {
    cursor_position: UVec2,
}

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
            CustomFastTileMapPlugin::<UserData> {
                // This is how you can insert custom code snippeds into tilemap_shader.wgsl.
                // Note that the code is inserted verbatim, so it requires some understanding of
                // the inner workings of the shader which may also change in the future.

                user_data_struct: Some(
                    r#"
                    cursor_position: vec2<u32>,
                    "#
                    .to_string(),
                ),

                // This code is inserted just before sampling a tile from the texture.
                // Use this for example to extract some extra information out of the tile index
                // or change the tile offset to be sampled.
                // Note that you can even declare variables here to refer to in `post_sample_code`,
                // as we do with "special".
                pre_sample_code: Some(
                    r#"
                    // Extract the "special bit" from the index
                    var special = (tile_index & 0x0100) != 0;

                    // For actual tile index we only need the lowest byte
                    tile_index = tile_index & 0x00FF;

                    // Also have the special tiles "bounce" up and down
                    if special {
                        tile_offset.y += abs(sin(animation_state * 5.0 + tile_offset.x * 0.002)) * 20.0;
                    }

                    "#
                    .to_string(),
                ),

                // After computation of the base color (the actual sampling step), we apply a red tint for "special" tiles.
                post_sample_code: Some(
                    r#"
                    if special {
                        // Special tiles are tinted red
                        color = color * vec4(1.0, 0.0, 0.0, 1.0);
                    }

                    if u32(tile_position.x) == user_data.cursor_position.x && u32(tile_position.y) == user_data.cursor_position.y {
                        // Highlight the hovered tile ("cursor position") with a white glow

                        var v = (sin(animation_state * 3.0) + 1.5) * (tile_offset.y + 64.0) / 40.0;
                        color = color * vec4(v, v, v, 1.0);
                    }
                    "#
                    .to_string(),
                ),
                ..default()
            },
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, highlight_hovered)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map<UserData>>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map = Map::<UserData>::builder(
        // Map size
        uvec2(100, 100),
        // Tile atlas
        asset_server.load("iso_256x128.png"),
        // Tile size
        vec2(256.0, 128.0),
    )
    .with_user_data(UserData {
        cursor_position: uvec2(50, 50),
    })
    .with_padding(vec2(256.0, 128.0), vec2(256.0, 128.0), vec2(256.0, 128.0))
    // "Perspective" overhang draws the overlap of tiles depending on their "depth" that is the
    // y-axis of their world position (tiles higher up are considered further away).
    .with_projection(AXONOMETRIC)
    .with_perspective_overhang()
    .build_and_initialize(init_map);

    commands.spawn(MapBundleManaged::<UserData> {
        material: materials.add(map),
        ..Default::default()
    });
} // startup

/// Fill the map with a random pattern
fn init_map(m: &mut MapIndexer<UserData>) {
    let mut rng = rand::thread_rng();
    for y in 0..m.size().y {
        for x in 0..m.size().x {
            // Actual tile index
            let mut v = rng.gen_range(1..4);

            // With a 10% chance, set the "special" bit, which
            // we will interpret in our custom shader code above
            if rng.gen_bool(0.1) {
                v |= 0x0100;
            }

            m.set(x, y, v);
        }
    }
}

/// Highlight the currently hovered tile red, reset all other tiles
fn highlight_hovered(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut camera_query: Query<(&GlobalTransform, &Camera), With<OrthographicProjection>>,
    maps: Query<&Handle<Map<UserData>>>,

    // We'll actually change the map contents for highlighting
    mut materials: ResMut<Assets<Map<UserData>>>,
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

                    let coord = coord
                        .as_uvec2()
                        .clamp(uvec2(0, 0), map.map_size() - uvec2(1, 1));

                    map.user_data.cursor_position = coord;
                } // if Some(world)
            } // for (global, camera)
        } // for map
    } // for event
} // highlight_hovered
