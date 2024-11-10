use std::ops::Range;

use crate::grid::grid::*;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    core_pipeline::{
        fxaa::Fxaa,
        prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass},
    },
    input::mouse::MouseWheel,
    pbr::ClusterConfig,
    prelude::*,
    render::view::{ColorGrading, ColorGradingGlobal, ColorGradingSection},
};

const KEYBOARD_PAN_SPEED: f32 = 10.0;
const KEYBOARD_ROTATE_SPEED: f32 = 1.0;
const MOUSE_PAN_SPEED: f32 = 5.0;
const MOUSE_ROTATE_SPEED: f32 = 0.25;

#[cfg(target_arch = "wasm32")]
const SCROLL_SPEED: f32 = 10.0;

#[cfg(not(target_arch = "wasm32"))]
const SCROLL_SPEED: f32 = 200.0;

#[derive(Component, Debug)]
pub struct PlayerCameraController {
    mouse_panning_last_position: Vec2,
    pub mouse_panning_in_progress: bool,
    mouse_rotating_last_position: Vec2,
    pub mouse_rotating_in_progress: bool,
    camera_center_ground_position: Vec3,
    pub keyboard_panning_in_progress: bool,
    pub keyboard_rotating_in_progress: bool,
}

impl PlayerCameraController {
    fn new() -> Self {
        Self {
            mouse_panning_last_position: Vec2::ZERO,
            mouse_panning_in_progress: false,
            mouse_rotating_last_position: Vec2::ZERO,
            mouse_rotating_in_progress: false,
            camera_center_ground_position: Vec3::ZERO,
            keyboard_panning_in_progress: false,
            keyboard_rotating_in_progress: false,
        }
    }
}

impl PlayerCameraController {
    pub fn is_moving(&self) -> bool {
        self.mouse_panning_in_progress
            || self.mouse_rotating_in_progress
            || self.keyboard_panning_in_progress
            || self.keyboard_rotating_in_progress
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
    let clear = Color::srgb(0.25, 0.25, 0.25);
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: false,
                clear_color: ClearColorConfig::Custom(clear),
                ..default()
            },
            tonemapping: Tonemapping::BlenderFilmic,
            color_grading: ColorGrading {
                highlights: ColorGradingSection {
                    contrast: 0.5,
                    gain: 0.5,
                    lift: 0.5,
                    gamma: 1.0,
                    saturation: 0.5,
                },
                global: ColorGradingGlobal {
                    exposure: 0.5,
                    hue: 0.5,
                    midtones_range: Range::from(0.1..1.0),
                    post_saturation: 0.5,
                    temperature: 1.0,
                    tint: 1.0,
                },
                midtones: ColorGradingSection {
                    contrast: 0.5,
                    gain: 0.5,
                    lift: 0.5,
                    gamma: 1.0,
                    saturation: 0.5,
                },
                shadows: ColorGradingSection {
                    contrast: 0.5,
                    gain: 0.5,
                    lift: 0.5,
                    gamma: 1.0,
                    saturation: 0.5,
                },
                ..default()
            },
            transform: Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FogSettings {
            color: clear.mix(&Color::BLACK, 0.2),
            directional_light_color: Color::srgba(1.0, 0.95, 0.85, 0.5),
            directional_light_exponent: 100.0,
            falloff: FogFalloff::from_visibility_colors(
                35.0,                       // distance in world units up to which objects retain visibility (>= 5% contrast)
                Color::srgb(0.5, 0.5, 0.6), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                Color::srgb(0.8, 0.8, 0.9), // atmospheric inscattering color (light gained due to scattering from the sun)
            ),
        },
        ClusterConfig::FixedZ {
            total: 4096,
            z_slices: 1,
            dynamic_resizing: true,
            z_config: Default::default(),
        },
        DepthPrepass,
        MotionVectorPrepass,
        DeferredPrepass,
        Fxaa::default(),
        BloomSettings::NATURAL,
        PlayerCameraController::new(),
    ));
}

fn keyboard_panning(
    mut query: Query<(&mut Transform, &mut PlayerCameraController)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut controller)) = query.get_single_mut() {
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

        controller.keyboard_panning_in_progress = delta != Vec3::ZERO;
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
                controller.mouse_panning_in_progress = true;
            }
        } else if mouse.just_released(MouseButton::Right) || (mouse.just_released(MouseButton::Left)) {
            controller.mouse_panning_in_progress = false;
        }

        if controller.mouse_panning_in_progress {
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
    mut query: Query<(&mut Transform, &mut PlayerCameraController)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut controller)) = query.get_single_mut() {
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
            controller.keyboard_rotating_in_progress = true;
        } else {
            controller.keyboard_rotating_in_progress = false;
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
                controller.mouse_rotating_in_progress = true;
            }
        } else if mouse.just_released(MouseButton::Middle) || (mouse.just_released(MouseButton::Left)) {
            controller.mouse_rotating_in_progress = false;
        }

        if controller.mouse_rotating_in_progress {
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

    let Ok(window) = windows.get_single() else {
        return;
    };

    let window_center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
    let Some(ray_center) = camera.viewport_to_world(camera_transform, window_center) else {
        return;
    };

    if let Some(center_distance) = ray_center.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up())) {
        let center_point = ray_center.get_point(center_distance);
        controller.camera_center_ground_position = center_point;
    };
}
