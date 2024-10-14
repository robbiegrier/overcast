use crate::{
    camera::PlayerCameraController,
    grid::{Grid, GridArea, GridCell, Ground},
    tool::ToolState,
};
use bevy::{math::VectorSpace, prelude::*};
use std::f32::consts::FRAC_PI_2;

pub struct RoadToolPlugin;

impl Plugin for RoadToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tool).add_systems(
            Update,
            (update_ground_position, (adjust_tool_size, change_orientation, handle_action))
                .run_if(in_state(ToolState::Road)),
        );
    }
}

#[derive(Component, Debug)]
pub struct Road;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum RoadToolMode {
    Spawner,
    Eraser,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum RoadOrientation {
    X,
    Z,
}

#[derive(Component, Debug)]
pub struct RoadTool {
    width: i32,
    ground_position: Vec3,
    drag_start_ground_position: Vec3,
    dragging: bool,
    drag_area: GridArea,
    mode: RoadToolMode,
    orientation: RoadOrientation,
}

impl RoadTool {
    fn new() -> Self {
        Self {
            width: 3,
            ground_position: Vec3::ZERO,
            drag_start_ground_position: Vec3::ZERO,
            dragging: false,
            drag_area: GridArea::at(Vec3::ZERO, 0, 0),
            mode: RoadToolMode::Spawner,
            orientation: RoadOrientation::Z,
        }
    }

    fn dimensions(&self) -> IVec2 {
        let mut length = 1;

        if self.dragging {
            length = match self.orientation {
                RoadOrientation::X => {
                    GridCell::at(self.ground_position).position.x - GridCell::at(self.drag_start_ground_position).position.x
                }
                RoadOrientation::Z => {
                    GridCell::at(self.ground_position).position.y - GridCell::at(self.drag_start_ground_position).position.y
                }
            }
        }

        length = length.abs();

        match self.orientation {
            RoadOrientation::X => IVec2::new(length, self.width),
            RoadOrientation::Z => IVec2::new(self.width, length),
        }
    }
}

fn spawn_tool(mut commands: Commands) {
    commands.spawn(RoadTool::new());
}

fn update_ground_position(
    camera_query: Query<(&Camera, &PlayerCameraController, &GlobalTransform)>,
    mut tool_query: Query<&mut RoadTool>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    grid_query: Query<&Grid>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, controller, camera_transform) = camera_query.single();
    let mut tool = tool_query.single_mut();
    let ground = ground_query.single();

    if let Some(cursor_position) = windows.single().cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            if let Some(distance) = ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up())) {
                let point = ray.get_point(distance);

                tool.ground_position = point;

                // let tool_dimensions = tool.dimensions();

                // let mut area = GridArea::at(tool.ground_position, tool_dimensions.x, tool_dimensions.y);

                let area: GridArea;
                if tool.dragging {
                    if tool.orientation == RoadOrientation::Z {
                        let start_area = GridArea::at(tool.drag_start_ground_position, tool.width, 1);
                        let project_drag_pos = tool.ground_position.with_x(tool.drag_start_ground_position.x);
                        gizmos.arrow(tool.drag_start_ground_position, project_drag_pos, Color::WHITE);
                        let end_area = GridArea::at(project_drag_pos, tool.width, 1);
                        area = start_area.union(end_area);
                    } else {
                        let start_area = GridArea::at(tool.drag_start_ground_position, 1, tool.width);
                        let project_drag_pos = tool.ground_position.with_z(tool.drag_start_ground_position.z);
                        gizmos.arrow(tool.drag_start_ground_position, project_drag_pos, Color::WHITE);
                        let end_area = GridArea::at(project_drag_pos, 1, tool.width);
                        area = start_area.union(end_area);
                    }
                } else {
                    if tool.orientation == RoadOrientation::Z {
                        area = GridArea::at(tool.ground_position, tool.width, 1);
                    } else {
                        area = GridArea::at(tool.ground_position, 1, tool.width);
                    }
                }

                tool.drag_area = area;

                let mut gizmo_color = match tool.mode {
                    RoadToolMode::Spawner => {
                        if grid_query.single().is_valid_paint_area(area) {
                            Color::linear_rgba(0.5, 0.0, 0.85, 0.8)
                        } else {
                            Color::linear_rgba(1.0, 0.0, 0.0, 0.25)
                        }
                    }
                    RoadToolMode::Eraser => Color::linear_rgba(1.0, 1.0, 0.0, 0.8),
                };

                if controller.is_moving() {
                    gizmo_color = gizmo_color.with_alpha(0.25);
                }

                gizmos.rect(
                    area.center() + ground.up() * 0.01,
                    Quat::from_rotation_x(FRAC_PI_2),
                    area.dimensions(),
                    gizmo_color,
                );
            }
        }
    }
}

fn adjust_tool_size(mut query: Query<&mut RoadTool>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut tool = query.single_mut();

    if keyboard.just_pressed(KeyCode::KeyR) {
        tool.width += 1;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        tool.width -= 1;
    }

    tool.width = tool.width.max(1);
}

fn change_orientation(mut query: Query<&mut RoadTool>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut tool = query.single_mut();

    if keyboard.just_pressed(KeyCode::Tab) {
        tool.orientation = match tool.orientation {
            RoadOrientation::X => RoadOrientation::Z,
            RoadOrientation::Z => RoadOrientation::X,
        }
    }
}

fn handle_action(
    commands: Commands,
    mut query: Query<&mut RoadTool>,
    mut grid_query: Query<&mut Grid>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut tool = query.single_mut();
    let mut grid = grid_query.single_mut();

    if mouse.just_pressed(MouseButton::Left) && !keyboard.any_pressed([KeyCode::AltLeft, KeyCode::ControlLeft]) {
        match tool.mode {
            RoadToolMode::Spawner => {
                if !tool.dragging {
                    handle_start_drag(&mut tool);
                } else {
                    handle_end_drag(commands, &mut tool, &mut grid, meshes, materials);
                }
            }
            RoadToolMode::Eraser => {}
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        tool.dragging = false;
    }
}

fn handle_start_drag(tool: &mut RoadTool) {
    tool.dragging = true;
    tool.drag_start_ground_position = tool.ground_position;
}

fn handle_end_drag(
    mut commands: Commands,
    tool: &mut RoadTool,
    grid: &mut Grid,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    tool.dragging = false;

    let road_height = 0.1;
    let road_color = 0.05;

    if grid.is_valid_paint_area(tool.drag_area) {
        let size = tool.drag_area.dimensions();
        let entity = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(size.x, road_height, size.y)),
                    material: materials.add(Color::linear_rgb(road_color, road_color, road_color)),
                    transform: Transform::from_translation(tool.drag_area.center().with_y(road_height / 2.0)),
                    ..default()
                },
                Road,
            ))
            .id();

        grid.mark_area_occupied(tool.drag_area, entity);
    }
}
