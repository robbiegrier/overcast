use bevy::{input::mouse::MouseWheel, prelude::*};

const KEYBOARD_PAN_SPEED: f32 = 10.0;
const MOUSE_PAN_SPEED: f32 = 2.5;
const SCROLL_SPEED: f32 = 100.0;

#[derive(Component, Debug)]
pub struct PlayerCameraController {
    mouse_panning_last_position: Vec2,
    panning_in_progress: bool,
}

impl PlayerCameraController {
    fn new() -> Self {
        Self {
            mouse_panning_last_position: Vec2::ZERO,
            panning_in_progress: false,
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start).add_systems(Update, (keyboard_panning, mouse_zoom, mouse_panning));
    }
}

fn start(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PlayerCameraController::new(),
    ));
}

fn keyboard_panning(
    mut query: Query<&mut Transform, With<PlayerCameraController>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let mut delta = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            delta += transform.forward().as_vec3().with_y(0.0).normalize();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            delta += transform.back().as_vec3().with_y(0.0).normalize();
        }
        if keyboard.pressed(KeyCode::KeyA) {
            delta += transform.left().as_vec3().with_y(0.0).normalize();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            delta += transform.right().as_vec3().with_y(0.0).normalize();
        }

        transform.translation += delta * KEYBOARD_PAN_SPEED * time.delta_seconds();
    }
}

fn mouse_zoom(
    mut query: Query<&mut Transform, With<PlayerCameraController>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let mut delta = Vec3::ZERO;

        for scroll in mouse_wheel.read() {
            delta += transform.forward().as_vec3() * scroll.y * SCROLL_SPEED * time.delta_seconds();
        }

        transform.translation += delta;
    }
}

fn mouse_panning(
    mut query: Query<(&mut Transform, &mut PlayerCameraController)>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut controller)) = query.get_single_mut() {
        if mouse.just_pressed(MouseButton::Right) {
            if let Some(cursor_position) = windows.single().cursor_position() {
                controller.mouse_panning_last_position = cursor_position;
                controller.panning_in_progress = true;
            }
        } else if mouse.just_released(MouseButton::Right) {
            controller.panning_in_progress = false;
        }

        if controller.panning_in_progress {
            if mouse.pressed(MouseButton::Right) {
                if let Some(cursor_position) = windows.single().cursor_position() {
                    let delta_mouse_drag = cursor_position - controller.mouse_panning_last_position;
                    let vertical = transform.forward().with_y(0.0).normalize() * delta_mouse_drag.y;
                    let horizontal = transform.left().with_y(0.0).normalize() * delta_mouse_drag.x;
                    let delta = (vertical + horizontal) * MOUSE_PAN_SPEED * time.delta_seconds();
                    transform.translation += delta;
                    controller.mouse_panning_last_position = cursor_position;
                }
            }
        }
    }
}
