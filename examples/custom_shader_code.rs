/*!
This example also demonstrates how to insert custom code into the tilemap shader to
modify the appearance of the tiles. In this case, we add a "special" bit to the tile index
and use it to make some tiles bounce up and down and tint them red.
*/

use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    window::PresentMode,
};

use bevy_fast_tilemap::prelude::*;
use rand::Rng;

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

#[derive(Debug, Clone, Default, Reflect, AsBindGroup, ShaderType)]
struct UserData {
    cursor_position: UVec2,
}

#[derive(Clone, TypePath, Default)]
struct MyCustomization;

impl Customization for MyCustomization {
    const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(0x1d1e1e1e1e1e1e1e);
    type UserData = UserData;

    // This is how you can insert custom code snippeds into tilemap_shader.wgsl.
    // Note that the code is inserted verbatim, so it requires some understanding of
    // the inner workings of the shader which may also change in the future.

    // If the below looks intimidating, it's because it does a lot of things:
    // - Defines a user data struct which holds the cursor position
    // - Extracts a "special" bit from the tile index
    // - Makes special tiles red & bounce up and down
    // - Mirrors tiles on the x-axis sometimes
    // - Adds a white glow to the hovered tile
    fn custom_shader_code() -> String {
        r#"
        // This is a custom user data struct that can be used in the shader code.
        // It is passed to the shader as a bind group, so it can be used to pass
        // additional information to the shader.
        struct UserData {
            cursor_position: vec2<u32>,
        };

        fn sample_tile(in: ExtractIn) -> vec4<f32> {

            // extract a "special" bit from the tile index and use it to
            // make some tiles bounce up and down.
            var special = (in.tile_index & 0x0100) != 0;

            // extract the actual tile index
            var tile_index = in.tile_index & 0x00FF;
            var tile_offset = in.tile_offset;

            if special {
                tile_offset.y += abs(sin(in.animation_state * 5.0 + tile_offset.x * 0.002)) * 20.0;
            }

            // Sometimes mirror tile on the x-Axis for some reason :)
            if user_data.cursor_position.x % 2 == 0 {
                tile_offset.x = 256.0 - tile_offset.x;
            }

            var color = sample_tile_at(tile_index, in.tile_position, tile_offset);

            // tint "special" tiles red
            if special {
                color = color * vec4(10.0, 0.0, 0.0, 1.0);
            }

            // Add a white glow to the hovered tile
            if u32(in.tile_position.x) == user_data.cursor_position.x && u32(in.tile_position.y) == user_data.cursor_position.y {
                var v = (sin(in.animation_state * 3.0) + 1.5) * (tile_offset.y + 64.0) / 40.0;
                color = color * vec4(v * 10.0, v * 10.0, v * 10.0, 1.0);
            }

            return color;
        }
    "#.to_string()
    }
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
            CustomFastTileMapPlugin::<MyCustomization>::default(),
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, update_cursor_position)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map<MyCustomization>>>,
) {
    // The bloom part is completely optional, we just do it to illustrate that
    // the standard bevy plugins can interact with the custom shader code.
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            ..default()
        },
        // Optional: To illustrate correct interaction with bevy plugins, we add a BloomSettings
        BloomSettings {
            composite_mode: BloomCompositeMode::Additive,
            ..Default::default()
        },
    ));

    let map = Map::<MyCustomization>::builder(
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

    commands.spawn(MapBundleManaged::<MyCustomization> {
        material: materials.add(map),
        ..Default::default()
    });
} // startup

/// Fill the map with a random pattern
fn init_map(m: &mut MapIndexerMut<MyCustomization>) {
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

/// Send cursor position to the shader via user data
fn update_cursor_position(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut camera_query: Query<(&GlobalTransform, &Camera), With<OrthographicProjection>>,
    maps: Query<&Handle<Map<MyCustomization>>>,

    // We'll actually change the map (by changing the user data), so we need to get a mutable
    mut materials: ResMut<Assets<Map<MyCustomization>>>,
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
