use crate::{
    building::Building,
    camera::PlayerCameraController,
    grid::{Grid, GridArea, Ground},
};
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::FRAC_PI_2;

pub struct BrushPlugin;

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_brush).add_systems(
            Update,
            (update_brush, adjust_brush_size, handle_brush_action, toggle_brush_mode),
        );
    }
}

#[derive(Debug)]
enum BrushMode {
    Building,
    Eraser,
}

#[derive(Component, Debug)]
pub struct Brush {
    dimensions: IVec2,
    ground_position: Vec3,
    mode: BrushMode,
}

impl Brush {
    fn new() -> Self {
        Self {
            dimensions: IVec2::ONE,
            ground_position: Vec3::ZERO,
            mode: BrushMode::Building,
        }
    }
}

fn spawn_brush(mut commands: Commands) {
    commands.spawn(Brush::new());
}

fn update_brush(
    camera_query: Query<(&Camera, &PlayerCameraController, &GlobalTransform)>,
    mut brush_query: Query<&mut Brush>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    grid_query: Query<&Grid>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, controller, camera_transform) = camera_query.single();
    let mut brush = brush_query.single_mut();
    let ground = ground_query.single();

    if let Some(cursor_position) = windows.single().cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            if let Some(distance) = ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up())) {
                let point = ray.get_point(distance);

                brush.ground_position = point;

                let area = GridArea::at(brush.ground_position, brush.dimensions.x, brush.dimensions.y);

                let mut gizmo_color = match brush.mode {
                    BrushMode::Building => Color::linear_rgba(0.0, 1.0, 1.0, 0.8),
                    BrushMode::Eraser => Color::linear_rgba(1.0, 1.0, 0.0, 0.8),
                };

                gizmo_color = match grid_query.single().is_valid_paint_area(area) {
                    true => gizmo_color,
                    false => Color::linear_rgba(1.0, 0.0, 0.0, 0.25),
                };

                if controller.is_moving() {
                    gizmo_color = gizmo_color.with_alpha(0.25);
                }

                gizmos.rounded_rect(
                    area.center() + ground.up() * 0.01,
                    Quat::from_rotation_x(FRAC_PI_2),
                    area.dimensions(),
                    gizmo_color,
                );
            }
        }
    }
}

fn toggle_brush_mode(mut query: Query<&mut Brush>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut brush = query.single_mut();

    if keyboard.just_pressed(KeyCode::Backspace) {
        brush.mode = match brush.mode {
            BrushMode::Building => BrushMode::Eraser,
            BrushMode::Eraser => BrushMode::Building,
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

fn handle_brush_action(
    commands: Commands,
    query: Query<&mut Brush>,
    mut grid_query: Query<&mut Grid>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let brush = query.single();
    let mut grid = grid_query.single_mut();

    if mouse.just_pressed(MouseButton::Left)
        && !keyboard.any_pressed([KeyCode::AltLeft, KeyCode::ShiftLeft, KeyCode::ControlLeft])
    {
        match brush.mode {
            BrushMode::Building => place_building(commands, brush, &mut grid, meshes, materials),
            BrushMode::Eraser => erase(commands, brush, &mut grid),
        }
    }
}

fn place_building(
    mut commands: Commands,
    brush: &Brush,
    grid: &mut Grid,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let area = GridArea::at(brush.ground_position, brush.dimensions.x, brush.dimensions.y);

    let rheight = rand::thread_rng().gen_range(1.0..5.0);
    let rgray = rand::thread_rng().gen_range(0.15..0.75);
    let alley_width = 0.1;

    if grid.is_valid_paint_area(area) {
        let size = area.dimensions();
        let entity = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(size.x - alley_width, rheight, size.y - alley_width)),
                    material: materials.add(Color::linear_rgb(rgray, rgray, rgray)),
                    transform: Transform::from_translation(area.center().with_y(rheight / 2.0)),
                    ..default()
                },
                Building,
            ))
            .id();

        grid.mark_area_occupied(area, entity);
    }
}

fn erase(mut commands: Commands, brush: &Brush, grid: &mut Grid) {
    let area = GridArea::at(brush.ground_position, brush.dimensions.x, brush.dimensions.y);

    for cell in area.iter() {
        if let Ok(entity_slot) = grid.entity_at(cell) {
            if let Some(entity) = entity_slot {
                println!("want to erase {:?}", entity);
                grid.erase(entity);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
