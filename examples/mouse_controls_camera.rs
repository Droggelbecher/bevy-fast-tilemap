
use bevy::{
    input::mouse::MouseWheel,
    math::vec3,
    prelude::*,
};

pub struct MouseControlsCameraPlugin;

impl Default for MouseControlsCameraPlugin {
    fn default() -> MouseControlsCameraPlugin {
        MouseControlsCameraPlugin {}
    }
}

impl Plugin for MouseControlsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseState::default())
            .add_system(mouse_controls_camera);
    }
}

#[derive(Default)]
struct MouseState {
    anchor: Vec2,
}

/// Use RMB for panning
/// Use scroll wheel for zooming
fn mouse_controls_camera(
    windows: Res<Windows>,
    mut mouse_state: ResMut<MouseState>,
    mouse_button: Res<Input<MouseButton>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    let win = windows.get_primary().expect("no primary window WTF?!");

    let mouse_pos = match win.cursor_position() {
        Some(mouse_pos) => mouse_pos,
        None => return,
    };

    if mouse_button.just_pressed(MouseButton::Right) {
        mouse_state.anchor = mouse_pos;
    }

    if mouse_button.pressed(MouseButton::Right) {
        let delta = mouse_pos - mouse_state.anchor;

        for (mut transform, mut _ortho) in camera_query.iter_mut() {
            transform.translation.x -= delta.x * transform.scale.x;
            transform.translation.y -= delta.y * transform.scale.y;
        }

        mouse_state.anchor = mouse_pos;
    }

    let mut wheel_y = 0.;
    for event in mouse_wheel_events.iter() {
        wheel_y += event.y;
    }

    if wheel_y != 0. {
        for (mut transform, mut _ortho) in camera_query.iter_mut() {
            let factor = f32::powf(2., -wheel_y / 2.);
            transform.scale *= vec3(factor, factor, 1.0);
            transform.scale = transform.scale.max(Vec3::splat(1. / 128.)).min(Vec3::splat(128.));
        }
    }
}
