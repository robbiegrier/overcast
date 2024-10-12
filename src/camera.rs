use crate::grid::*;
use bevy::{input::mouse::MouseWheel, prelude::*};

const KEYBOARD_PAN_SPEED: f32 = 10.0;
const KEYBOARD_ROTATE_SPEED: f32 = 1.0;
const MOUSE_PAN_SPEED: f32 = 5.0;
const MOUSE_ROTATE_SPEED: f32 = 0.25;
const SCROLL_SPEED: f32 = 100.0;

#[derive(Component, Debug)]
pub struct PlayerCameraController {
    mouse_panning_last_position: Vec2,
    panning_in_progress: bool,
    mouse_rotating_last_position: Vec2,
    rotating_in_progress: bool,
    camera_center_ground_position: Vec3,
}

impl PlayerCameraController {
    fn new() -> Self {
        Self {
            mouse_panning_last_position: Vec2::ZERO,
            panning_in_progress: false,
            mouse_rotating_last_position: Vec2::ZERO,
            rotating_in_progress: false,
            camera_center_ground_position: Vec3::ZERO,
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            (
                update_camera_raycast,
                (keyboard_panning, mouse_zoom, mouse_panning, keyboard_rotating, mouse_rotating),
            ),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
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
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut controller)) = query.get_single_mut() {
        if mouse.just_pressed(MouseButton::Right)
            || (mouse.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::AltLeft))
        {
            if let Some(cursor_position) = windows.single().cursor_position() {
                controller.mouse_panning_last_position = cursor_position;
                controller.panning_in_progress = true;
            }
        } else if mouse.just_released(MouseButton::Right) || (mouse.just_released(MouseButton::Left)) {
            controller.panning_in_progress = false;
        }

        if controller.panning_in_progress {
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

fn keyboard_rotating(
    mut query: Query<(&mut Transform, &PlayerCameraController)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, controller)) = query.get_single_mut() {
        let mut delta_angle = 0.0f32;

        if keyboard.pressed(KeyCode::KeyQ) {
            delta_angle += KEYBOARD_ROTATE_SPEED;
        }
        if keyboard.pressed(KeyCode::KeyE) {
            delta_angle -= KEYBOARD_ROTATE_SPEED;
        }

        if delta_angle != 0.0 {
            let rotate_point = controller.camera_center_ground_position.with_y(transform.translation.y);
            let quat = Quat::from_rotation_y(delta_angle * time.delta_seconds());
            transform.rotate_around(rotate_point, quat);
        }
    }
}

fn mouse_rotating(
    mut query: Query<(&mut Transform, &mut PlayerCameraController)>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut controller)) = query.get_single_mut() {
        if mouse.just_pressed(MouseButton::Middle)
            || (mouse.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ControlLeft))
        {
            if let Some(cursor_position) = windows.single().cursor_position() {
                controller.mouse_rotating_last_position = cursor_position;
                controller.rotating_in_progress = true;
            }
        } else if mouse.just_released(MouseButton::Middle) || (mouse.just_released(MouseButton::Left)) {
            controller.rotating_in_progress = false;
        }

        if controller.rotating_in_progress {
            if let Some(cursor_position) = windows.single().cursor_position() {
                let delta_mouse_drag = cursor_position - controller.mouse_rotating_last_position;

                let quat_horizontal = Quat::from_rotation_y(-delta_mouse_drag.x * MOUSE_ROTATE_SPEED * time.delta_seconds());
                let quat_vertical = Quat::from_axis_angle(
                    transform.right().as_vec3(),
                    -delta_mouse_drag.y * MOUSE_ROTATE_SPEED * time.delta_seconds(),
                );
                let rotate_point = controller.camera_center_ground_position.with_y(transform.translation.y);

                transform.rotate_around(controller.camera_center_ground_position, quat_vertical);
                transform.rotate_around(rotate_point, quat_horizontal);

                controller.mouse_rotating_last_position = cursor_position;
            }
        }
    }
}

fn update_camera_raycast(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut controller_query: Query<&mut PlayerCameraController>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
) {
    let (camera, camera_transform) = camera_query.single();
    let mut controller = controller_query.single_mut();
    let ground = ground_query.single();

    let window_center = Vec2::new(windows.single().width() / 2.0, windows.single().height() / 2.0);
    if let Some(ray_center) = camera.viewport_to_world(camera_transform, window_center) {
        if let Some(center_distance) = ray_center.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up())) {
            let center_point = ray_center.get_point(center_distance);
            controller.camera_center_ground_position = center_point;
        };
    };
}
