//! This example demonstrates how you can use a different custom shader for each layer in your map.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2, vec4},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    window::PresentMode,
};

use bevy_fast_tilemap::{
    bundle::MapBundleManaged, map::MapIndexer, plugin::Customization, CustomFastTileMapPlugin, Map,
};
use rand::Rng;

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

#[derive(Debug, Clone, Default, Reflect, AsBindGroup, ShaderType)]
struct UserDataA {
    color: Vec4,
}

#[derive(Clone, TypePath, Default)]
struct CustomizationA;
impl Customization for CustomizationA {
    const CUSTOM_SHADER_CODE: &'static str = r#"
        // This is a custom user data struct that can be used in the shader code.
        // It is passed to the shader as a bind group, so it can be used to pass
        // additional information to the shader.
        struct UserData {
            color: vec4<f32>,
        };

        fn sample_tile(in: ExtractIn) -> vec4<f32> {
            if in.tile_index == 0 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            return user_data.color * sample_tile_at(in.tile_index, in.tile_position, in.tile_offset);
        }
    "#;
    const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(0x1d1e1e1e1e1e1e1e);
    type UserData = UserDataA;
}

#[derive(Debug, Clone, Default, Reflect, AsBindGroup, ShaderType)]
struct UserDataB {
    frequency: f32,
}

#[derive(Clone, TypePath, Default)]
struct CustomizationB;
impl Customization for CustomizationB {
    const CUSTOM_SHADER_CODE: &'static str = r#"
        struct UserData {
            frequency: f32,
        };

        fn sample_tile(in: ExtractIn) -> vec4<f32> {
            if in.tile_index == 0 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            var offs = in.tile_offset;
            let r = 1.5;
            offs.x += sin(f32(in.tile_offset.y) + user_data.frequency * in.animation_state) * r;
            offs.y += cos(f32(in.tile_offset.x) + user_data.frequency * in.animation_state) * r;
            return sample_tile_at(in.tile_index, in.tile_position, offs);
        }
    "#;
    const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(0x1d1e1e1e1e1e1e1f);
    type UserData = UserDataB;
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
            CustomFastTileMapPlugin::<CustomizationA>::default(),
            CustomFastTileMapPlugin::<CustomizationB>::default(),
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials_a: ResMut<Assets<Map<CustomizationA>>>,
    mut materials_b: ResMut<Assets<Map<CustomizationB>>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Shader A
    // Tint everything green

    let map = Map::<CustomizationA>::builder(
        uvec2(100, 100),
        asset_server.load("debug01.png"),
        vec2(64.0, 64.0),
    )
    .with_user_data(UserDataA {
        color: vec4(0.0, 1.5, 0.0, 1.0),
    })
    .build_and_initialize(init_map);

    commands.spawn(MapBundleManaged::<CustomizationA> {
        material: materials_a.add(map),
        transform: Transform::from_translation(Vec3::new(-100.0, -100.0, 20.0)),
        ..Default::default()
    });

    // Shader B
    // Add a wobble effect to the tiles

    let map = Map::<CustomizationB>::builder(
        uvec2(100, 100),
        asset_server.load("debug01.png"),
        vec2(64.0, 64.0),
    )
    .with_user_data(UserDataB {
        frequency: 5.0,
    })
    .build_and_initialize(init_map);

    commands.spawn(MapBundleManaged::<CustomizationB> {
        material: materials_b.add(map),
        transform: Transform::from_translation(Vec3::new(100.0, 100.0, 10.0)),
        ..Default::default()
    });

} // startup

/// Fill the map with a random pattern
fn init_map<C: Customization>(m: &mut MapIndexer<C>) {
    let mut rng = rand::thread_rng();
    for y in 0..m.size().y {
        for x in 0..m.size().x {
            let v = rng.gen_range(0..2);
            m.set(x, y, v);
        }
    }
}
