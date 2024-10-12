use crate::grid::{Grid, GridArea, Ground};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub struct BrushPlugin;

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_brush).add_systems(Update, (update_brush, adjust_brush_size, handle_paint));
    }
}

#[derive(Component, Debug)]
pub struct Brush {
    dimensions: IVec2,
    ground_position: Vec3,
}

impl Brush {
    fn new() -> Self {
        Self {
            dimensions: IVec2::ONE,
            ground_position: Vec3::ZERO,
        }
    }
}

fn spawn_brush(mut commands: Commands) {
    commands.spawn(Brush::new());
}

fn update_brush(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut brush_query: Query<&mut Brush>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let mut brush = brush_query.single_mut();
    let ground = ground_query.single();

    if let Some(cursor_position) = windows.single().cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            if let Some(distance) = ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up())) {
                let point = ray.get_point(distance);

                brush.ground_position = point;

                let area = GridArea::at(brush.ground_position, brush.dimensions.x, brush.dimensions.y);

                gizmos.rounded_rect(
                    area.center() + ground.up() * 0.01,
                    Quat::from_rotation_x(FRAC_PI_2),
                    area.dimensions(),
                    Color::linear_rgba(0.0, 1.0, 1.0, 1.0),
                );
            }
        }
    }
}

fn adjust_brush_size(mut query: Query<&mut Brush>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut brush = query.single_mut();

    if keyboard.just_pressed(KeyCode::KeyR) {
        brush.dimensions.x += 1;
        brush.dimensions.y += 1;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        brush.dimensions.x -= 1;
        brush.dimensions.y -= 1;
    }

    if keyboard.just_pressed(KeyCode::BracketRight) {
        brush.dimensions.x += 1;
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        brush.dimensions.x -= 1;
    }

    if keyboard.just_pressed(KeyCode::Equal) {
        brush.dimensions.y += 1;
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        brush.dimensions.y -= 1;
    }

    brush.dimensions = brush.dimensions.max(IVec2::new(1, 1));
}

fn handle_paint(
    query: Query<&mut Brush>,
    mut grid_query: Query<&mut Grid>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let brush = query.single();
    let mut grid = grid_query.single_mut();

    if mouse.pressed(MouseButton::Left)
        && !keyboard.any_pressed([KeyCode::AltLeft, KeyCode::ShiftLeft, KeyCode::ControlLeft])
    {
        let area = GridArea::at(brush.ground_position, brush.dimensions.x, brush.dimensions.y);

        for cell in area.iter() {
            if let Some(occupancy) = grid.is_occupied(cell) {
                if !occupancy {
                    println!("marking {:?}", cell);
                    grid.mark_occupied(cell);
                } else {
                    println!("already marked {:?}", cell);
                }
            } else {
                println!("Cannot mark {:?}, out of bounds", cell);
            }
        }
    }
}
