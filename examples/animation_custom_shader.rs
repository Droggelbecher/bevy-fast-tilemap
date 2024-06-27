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
use bevy_fast_tilemap::{
    map::DefaultUserData, plugin::Customization, CustomFastTileMapPlugin, Map, MapBundleManaged
};

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

#[derive(Clone, TypePath, Default)]
struct AnimationCustomization;
impl Customization for AnimationCustomization {
    const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(0x1d1e1e1e1e1e1e1e);
    type UserData = DefaultUserData;
    const CUSTOM_SHADER_CODE: &'static str = r#"
        // wgpu doesnt like this being empty, so the default is to have a dummy u32
        // field here.

        struct UserData {
            dummy: u32,
        };

        fn sample_tile(in: ExtractIn) -> vec4<f32> {
            var tile_index = in.tile_index;

            // If the map data says tile 6, animate it by changing the tile index to 6, 7 or 8
            // based on the animation state.
            if tile_index == 6u {
                var offs = u32(round(in.animation_state * 5.0)) % 3u;
                tile_index = 6u + offs;
            }

            return sample_tile_at(tile_index, in.tile_position, in.tile_offset);
        }
    "#;
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
            CustomFastTileMapPlugin::<AnimationCustomization>::default(),
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map<AnimationCustomization>>>,
) {
    commands.spawn(Camera2dBundle::default());

    // background tiles layer

    let map = Map::builder(
        uvec2(64, 64),
        asset_server.load("pixel_tiles_16.png"),
        vec2(16., 16.),
    )
    .build_and_set(|p| ((p.x + p.y) % 4 + 1) as u32);

    commands.spawn(MapBundleManaged::<AnimationCustomization> {
        material: materials.add(map),
        ..default()
    });

    // animated layer

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
        // set positive z value so this is rendered on top of the background layer
        transform: Transform::default().with_translation(vec3(0., 0., 1.)),
        ..default()
    };

    commands.spawn(bundle);
}
