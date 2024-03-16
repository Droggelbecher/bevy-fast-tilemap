//! This example also demonstrates how to insert custom code into the tilemap shader to
//! modify the appearance of the tiles. In this case, we add a "special" bit to the tile index
//! and use it to make some tiles bounce up and down and tint them red.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{ivec2, uvec2, vec2},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    window::PresentMode,
};

use bevy_fast_tilemap::{
    bundle::MapBundleManaged, map::MapIndexer, FastTileMapPlugin, Map, MapAttributes, AXONOMETRIC,
};
use rand::Rng;

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

#[derive(Debug, Clone, Default, Reflect, AsBindGroup, ShaderType)]
struct UserData {
    cursor_position: IVec2,
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
            FastTileMapPlugin::<UserData> {
                // This is how you can insert custom code snippeds into tilemap_shader.wgsl.
                // Note that the code is inserted verbatim, so it requires some understanding of
                // the inner workings of the shader which may also change in the future.

                user_data_struct: Some(
                    r#"
                    cursor_position: vec2<i32>,
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

                    if tile_position.x == user_data.cursor_position.x && tile_position.y == user_data.cursor_position.y {
                        color = color * vec4(10.0, 10.0, 10.0, 1.0);
                    }
                    "#
                    .to_string(),
                ),
                ..default()
            },
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, touch_map_attributes)
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
        cursor_position: ivec2(50, 50),
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

fn touch_map_attributes(mut map_query: Query<&mut MapAttributes>) {
    for mut map in map_query.iter_mut() {
        // You can of course set your own animation state here,
        // which can also be different per vertex (and will be interpolated, like all vertex
        // attributes).
        //map.animation_state += time.delta_seconds();

        // Fast-tilemap provides a default animation_state which is time.elapsed_seconds_wrapped().
        // We still need to mark the attributes changed every frame to trigger a re-upload of the vertices to the GPU,
        // if we intend to use this timing information.
        map.as_mut();
    }
}
