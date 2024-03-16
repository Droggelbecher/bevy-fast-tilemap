//! Like animation.rs, but instead of changing the map data every 200ms,
//! we insert some custom shader code that changes the displayed tile on GPU only.
//! This has the following implications:
//! - The map is not re-uploaded to the GPU (faster for very big maps)
//! - The bevy-side map does not reflect the current animation state (which may or may not be
//!     desired, depending on your application). This could be worked around by computing
//!     the animation state again on CPU when needed.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2, vec3},
    prelude::*,
    window::PresentMode,
};
use bevy_fast_tilemap::{FastTileMapPlugin, Map, MapAttributes, MapBundleManaged};

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
            FastTileMapPlugin {
                pre_sample_code: Some(
                    r#"

                    // If the map data says tile 6, animate it by changing the tile index to 6, 7 or 8
                    // based on the animation state.
                    if tile_index == 6u {
                        var offs = u32(round(animation_state * 5.0)) % 3u;
                        tile_index = 6u + offs;
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

#[derive(Component)]
struct AnimationLayer;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map = Map::builder(
        uvec2(64, 64),
        asset_server.load("pixel_tiles_16.png"),
        vec2(16., 16.),
    )
    .build_and_set(|p| ((p.x + p.y) % 4 + 1) as u32);

    commands.spawn(MapBundleManaged {
        material: materials.add(map),
        ..default()
    });

    let (l, h) = (32 - 10, 32 + 10);

    let map = Map::builder(
        uvec2(64, 64),
        asset_server.load("pixel_tiles_16.png"),
        vec2(16., 16.),
    )
    .build_and_set(|p| {
        if p.x >= l && p.x <= h && p.y >= l && p.y <= h {
            6
        } else {
            0
        }
    });

    let bundle = MapBundleManaged {
        material: materials.add(map),
        transform: Transform::default().with_translation(vec3(0., 0., 1.)),
        ..default()
    };

    commands.spawn(bundle).insert(AnimationLayer);
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
