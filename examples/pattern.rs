use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{uvec2, vec2},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    window::PresentMode,
};
use bevy_fast_tilemap::{plugin::Customization, CustomFastTileMapPlugin, Map, MapBundleManaged};
use rand::Rng;

#[path = "common/mouse_controls_camera.rs"]
mod mouse_controls_camera;
use mouse_controls_camera::MouseControlsCameraPlugin;

#[derive(Debug, Clone, Default, Reflect, AsBindGroup, ShaderType)]
struct UserData {
    x: u32,
}

#[derive(Clone, TypePath, Default)]
struct PatternCustomization;
impl Customization for PatternCustomization {
    const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(0x1d1e1e1e1e1e1e1e);
    type UserData = UserData;
    const CUSTOM_SHADER_CODE: &'static str = r#"
        struct UserData {
            x: u32,
        };

        // Simple 1d function that defines the irregular boundary shape
        // Returns values in the range -1.0 to 1.0
        fn offs_rocky(x: f32) -> f32 {
            var pi = radians(180.0);
            return 1.25 * (
                    0.1 * sin(x / 3.0)
                + 0.1 * sin((x+0.1) / 3.2)
                + 0.1 * sin((x+0.2) / 2.8)
                + 0.2 * sin((x+1) / 1.0)
                + 0.1 * sin((x+2.1) / 0.5)
                + 0.1 * sin((x+2.2) / 0.4)
                + 0.1 * sin((x+2.3) / 0.3));
        }

        // Scaled version.
        // Returns values in the range -6.0 to 6.0
        fn offs(x: f32) -> f32 {
            return 6.0 * offs_rocky(x / 6.0);
        }

        // DEBUG: Render tiles flat
        fn dbg_sample_tile(in: ExtractIn) -> vec4<f32> {
            return sample_tile_at(in.tile_index, in.tile_position, in.tile_offset);
        }

        fn sample_tile(in: ExtractIn) -> vec4<f32> {
            // offs() is shifted this much inwards
            // Bigger means smaller tiles and more fluid borders with eg diagonal
            // neighbors.
            //
            // doffs >= offs() + bw
            // must be fulfilled so that the border is drawn correctly.
            var doffs = 8.0;

            // half the border width
            var bw = 2.0;

            // debug borders
            var bw_dbg = -1.0;

            // Color of the border
            var border_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

            // How fast should the border move? (0.0 = no movement)
            // This makes for a more psychedelic effect and can be pretty distracting.
            var animation_speed = 0.0;

            // DEBUG: Render rectangular tile borders
            if in.tile_offset.x <= 2.0 * bw_dbg || in.tile_offset.y <= 2.0 * bw_dbg {
                return vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }

            var tile_index = in.tile_index;
            var a = in.animation_state * animation_speed;

            // shift between left/right and top/bottom offs() calls.
            // This way opposing sides of a tile don't look exactly the same.
            var shift = 100.0;

            // Positions of the (inner) irregular boundaries inside this tile
            var r = map.tile_size.x + offs(f32(in.tile_position.y) * map.tile_size.y + in.tile_offset.y + a) - doffs;
            var l = offs(f32(in.tile_position.y) * map.tile_size.y + in.tile_offset.y - a + shift) + doffs;
            var t = map.tile_size.y + offs(f32(in.tile_position.x) * map.tile_size.x + in.tile_offset.x - a) - doffs;
            var b = offs(f32(in.tile_position.x) * map.tile_size.x + in.tile_offset.x + a + shift) + doffs;

            // determine actual tile index to render at this position
            var tile_offset = vec2<i32>(0, 0);
            if in.tile_offset.x > r - bw {
                tile_offset = tile_offset + vec2<i32>(1, 0);
            }
            else if in.tile_offset.x <= l + bw {
                tile_offset = tile_offset + vec2<i32>(-1, 0);
            }
            if in.tile_offset.y > t - bw {
                tile_offset = tile_offset + vec2<i32>(0, 1);
            }
            else if in.tile_offset.y <= b + bw {
                tile_offset = tile_offset + vec2<i32>(0, -1);
            }
            var tile_position = in.tile_position + tile_offset;
            tile_index = get_tile_index_checked(tile_position);

            // Dominance rule: highest tile index wins.
            // I.e. high tile indices will form diagonal connections at the expense of
            // lower indices.
            var max_index = max(in.tile_index, tile_index);

            // Figure out border drawing.
            // Borders are drawn only between tiles of *different indices*.
            // In the easy case the border is between "this" tile index (in.tile_index)
            // and some horizontal/vertical neighbor.
            //
            // It might however also be between a horizontal/vertical neighbor and a
            // diagonal neighbor, in which case it has nothing to do with in.tile_index.
            // In that case we need to compare our neighbors indices with each other.

            var is_border = false;

            var is_xborder = abs(in.tile_offset.x - r) <= bw || abs(in.tile_offset.x - l) <= bw;
            var is_yborder = abs(in.tile_offset.y - t) <= bw || abs(in.tile_offset.y - b) <= bw;

            // DEBUG: Render offs() instances
            if abs(in.tile_offset.x - r) <= bw_dbg || abs(in.tile_offset.x - l) <= bw_dbg {
                return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }
            if abs(in.tile_offset.y - t) <= bw_dbg || abs(in.tile_offset.y - b) <= bw_dbg {
                return vec4<f32>(0.0, 1.0, 1.0, 1.0);
            }

            // diagonal case
            if tile_offset.x != 0 && tile_offset.y != 0 {

                //    ##===+===+===##===========##
                //    ||   |   |   ||           ||
                //    ||---+---+---||           ||
                //    ||   | C |   ||     R     ||
                //    ||---+---+---||           ||
                //    ||   |   | d ||           ||
                //    ##===+===+===##===========##
                //    ||           ||           ||
                //    ||           ||           ||
                //    ||     B     ||     D     ||
                //    ||           ||           ||
                //    ||           ||           ||
                //    ##===========##===========##
                //

                // The dominating tile index in a diagonal sector (d) is the max index of
                // the current tile (C) and the three neighbors (R, B, D).
                // `max_index` at this point is already max(C, D), so we only need to
                // include R & B.
                max_index = max(max_index, get_tile_index_checked(in.tile_position + vec2<i32>(tile_offset.x, 0)));
                max_index = max(max_index, get_tile_index_checked(in.tile_position + vec2<i32>(0, tile_offset.y)));

                if is_xborder && max(in.tile_index, get_tile_index_checked(in.tile_position + vec2<i32>(0, tile_offset.y))) != max_index {
                    // The "|" left of (d) in the picture above
                    is_border = true;
                }
                if is_yborder && max(in.tile_index, get_tile_index_checked(in.tile_position + vec2<i32>(tile_offset.x, 0))) != max_index {
                    // The "-" above (d) in the picture above
                    is_border = true;
                }
                if is_xborder && is_yborder && max_index > in.tile_index {
                    // The little "+" between (C) and (d) in the picture above
                    is_border = true;
                }
            }
            else if (is_xborder || is_yborder) && max_index > in.tile_index {
                is_border = true;
            }

            if is_border
            {
                return border_color;
            }

            var color = sample_tile_at(max_index, in.tile_position, in.tile_offset);
            return color;
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
            CustomFastTileMapPlugin::<PatternCustomization>::default(),
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map<PatternCustomization>>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map = Map::<PatternCustomization>::builder(
        uvec2(16, 16),
        asset_server.load("patterns.png"),
        vec2(64., 64.),
    )
    // A pattern texture in the atlas is NxN map tiles large
    // (i.e. it will repeat every N tiles in any direction)
    .with_atlas_tile_size_factor(4)
    // bevy_fast_tilemap will automatically calculate the correct texture coordinates so that
    // adjecent tiles in the map will also have adjecent texture coordinates.
    .build_and_initialize(|m| {
        let mut rng = rand::thread_rng();

        for y in 0..m.size().y {
            for x in 0..m.size().y {
                m.set(x, y, rng.gen_range(0..4));
            }
        }
    });

    commands.spawn(MapBundleManaged {
        material: materials.add(map),
        ..default()
    });
}
