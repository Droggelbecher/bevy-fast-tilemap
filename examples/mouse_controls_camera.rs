
//! Helper module for examples, allowing panning and zooming with the mouse.

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
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
        app.add_system(mouse_controls_camera);
    }
}

/// Use RMB for panning
/// Use scroll wheel for zooming
fn mouse_controls_camera(
    mouse_button: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&GlobalTransform, &mut Transform, &Camera, &mut OrthographicProjection)>,
) {

    for event in mouse_motion_events.iter() {
        if mouse_button.pressed(MouseButton::Left) || mouse_button.pressed(MouseButton::Right) {
            for (_, mut transform, _, _) in camera_query.iter_mut() {
                transform.translation.x -= event.delta.x * transform.scale.x;
                transform.translation.y += event.delta.y * transform.scale.y;
            }
        }
    }

    let mut wheel_y = 0.;
    for event in mouse_wheel_events.iter() {
        wheel_y += event.y;
    }

    if wheel_y != 0. {
        for (_, mut transform, _, mut _ortho) in camera_query.iter_mut() {
            let factor = f32::powf(2., -wheel_y / 2.);
            transform.scale *= vec3(factor, factor, 1.0);
            transform.scale = transform.scale.max(Vec3::splat(1. / 128.)).min(Vec3::splat(128.));
        }
    }
}
