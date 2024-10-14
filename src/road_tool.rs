use crate::{
    camera::PlayerCameraController,
    grid::{Grid, GridArea, GridCell, Ground},
    tool::ToolState,
};
use bevy::{ecs::observer::TriggerTargets, prelude::*};
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
pub struct RoadSegment {
    orientation: RoadOrientation,
    width: i32,
    area: GridArea,
}

#[derive(Component, Debug)]
pub struct Intersection {
    area: GridArea,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum RoadToolMode {
    Spawner,
    Eraser,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum RoadOrientation {
    #[default]
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

    fn area(&self) -> GridArea {
        if self.dragging {
            self.drag_start_area().union(self.drag_end_area())
        } else {
            self.hover_area()
        }
    }

    fn drag_start_area(&self) -> GridArea {
        if self.orientation == RoadOrientation::Z {
            GridArea::at(self.drag_start_ground_position, self.width, 1)
        } else {
            GridArea::at(self.drag_start_ground_position, 1, self.width)
        }
    }

    fn drag_end_area(&self) -> GridArea {
        if self.orientation == RoadOrientation::Z {
            GridArea::at(self.ground_position.with_x(self.drag_start_ground_position.x), self.width, 1)
        } else {
            GridArea::at(self.ground_position.with_z(self.drag_start_ground_position.z), 1, self.width)
        }
    }

    fn hover_area(&self) -> GridArea {
        if self.orientation == RoadOrientation::Z {
            GridArea::at(self.ground_position, self.width, 1)
        } else {
            GridArea::at(self.ground_position, 1, self.width)
        }
    }

    fn drag_start_attach_area(&self) -> GridArea {
        let start = self.drag_start_area();
        let end = self.drag_end_area();

        if self.orientation == RoadOrientation::Z {
            if end.max.position.y >= start.max.position.y {
                start.adjacent_bottom()
            } else {
                start.adjacent_top()
            }
        } else {
            if end.max.position.x >= start.max.position.x {
                start.adjacent_left()
            } else {
                start.adjacent_right()
            }
        }
    }

    fn drag_end_attach_area(&self) -> GridArea {
        let start = self.drag_start_area();
        let end = self.drag_end_area();

        if self.orientation == RoadOrientation::Z {
            if end.max.position.y >= start.max.position.y {
                end.adjacent_top()
            } else {
                end.adjacent_bottom()
            }
        } else {
            if end.max.position.x >= start.max.position.x {
                end.adjacent_right()
            } else {
                end.adjacent_left()
            }
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

                let area = tool.area();

                if tool.dragging {
                    tool.drag_area = area;
                }

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
    segment_query: Query<&mut RoadSegment>,
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
                    handle_end_drag(commands, &mut tool, &mut grid, meshes, materials, segment_query);
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
    segment_query: Query<&mut RoadSegment>,
) {
    if grid.is_valid_paint_area(tool.drag_area) {
        if let Some(adjacent_entity) = grid.single_entity_in_area(tool.drag_start_attach_area()) {
            if let Ok(adjacent_road_segment) = segment_query.get(adjacent_entity) {
                if adjacent_road_segment.orientation != tool.orientation {
                    println!("at start, create intersection");

                    grid.erase(adjacent_entity);

                    if adjacent_road_segment.orientation == RoadOrientation::Z {
                        split_road_z(adjacent_road_segment, &mut commands, tool, grid, &mut meshes, &mut materials);
                    } else {
                        split_road_x(adjacent_road_segment, &mut commands, tool, grid, &mut meshes, &mut materials);
                    }

                    commands.entity(adjacent_entity).despawn();
                } else if adjacent_road_segment.width == tool.width {
                    println!("at start, create extension");
                }
            }
        } else {
            println!("at start, new road");
        }

        if let Some(adjacent_entity) = grid.single_entity_in_area(tool.drag_end_attach_area()) {
            if let Ok(adjacent_road_segment) = segment_query.get(adjacent_entity) {
                if adjacent_road_segment.orientation != tool.orientation {
                    println!("at end, create intersection");

                    grid.erase(adjacent_entity);

                    if adjacent_road_segment.orientation == RoadOrientation::Z {
                        split_road_z(adjacent_road_segment, &mut commands, tool, grid, &mut meshes, &mut materials);
                    } else {
                        split_road_x(adjacent_road_segment, &mut commands, tool, grid, &mut meshes, &mut materials);
                    }

                    commands.entity(adjacent_entity).despawn();
                } else if adjacent_road_segment.width == tool.width {
                    println!("at end, create extension");
                }
            }
        } else {
            println!("at end, new road");
        }

        spawn_road_segment(
            tool.orientation,
            tool.width,
            tool.drag_area,
            grid,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }

    tool.dragging = false;
}

fn spawn_road_segment(
    orientation: RoadOrientation,
    width: i32,
    area: GridArea,
    grid: &mut Grid,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let size = area.dimensions();
    let road_height = 0.05;
    let road_color = if orientation == RoadOrientation::Z { 0.05 } else { 0.1 };

    let entity = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(size.x, road_height, size.y)),
                material: materials.add(Color::linear_rgb(road_color, road_color, road_color)),
                transform: Transform::from_translation(area.center().with_y(road_height / 2.0)),
                ..default()
            },
            RoadSegment {
                orientation: orientation,
                width: width,
                area: area,
            },
        ))
        .id();

    grid.mark_area_occupied(area, entity);
}

fn spawn_intersection(
    area: GridArea,
    grid: &mut Grid,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let size = area.dimensions();
    let road_height = 0.05;

    let entity = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(size.x, road_height, size.y)),
                material: materials.add(Color::linear_rgb(0.0, 0.1, 0.3)),
                transform: Transform::from_translation(area.center().with_y(road_height / 2.0)),
                ..default()
            },
            Intersection { area: area },
        ))
        .id();

    grid.mark_area_occupied(area, entity);
}

fn split_road_z(
    adjacent_road_segment: &RoadSegment,
    mut commands: &mut Commands,
    tool: &mut RoadTool,
    grid: &mut Grid,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    if adjacent_road_segment.area.min.position.y < tool.drag_area.min.position.y {
        spawn_road_segment(
            adjacent_road_segment.orientation,
            adjacent_road_segment.width,
            GridArea {
                min: adjacent_road_segment.area.min,
                max: GridCell::new(
                    adjacent_road_segment.area.max.position.x,
                    tool.drag_area.adjacent_bottom().min.position.y,
                ),
            },
            grid,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }

    if adjacent_road_segment.area.max.position.y > tool.drag_area.max.position.y {
        spawn_road_segment(
            adjacent_road_segment.orientation,
            adjacent_road_segment.width,
            GridArea {
                min: GridCell::new(
                    adjacent_road_segment.area.min.position.x,
                    tool.drag_area.adjacent_top().max.position.y,
                ),
                max: adjacent_road_segment.area.max,
            },
            grid,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }

    spawn_intersection(
        GridArea {
            min: GridCell::new(adjacent_road_segment.area.min.position.x, tool.drag_area.min.position.y),
            max: GridCell::new(adjacent_road_segment.area.max.position.x, tool.drag_area.max.position.y),
        },
        grid,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}

fn split_road_x(
    adjacent_road_segment: &RoadSegment,
    mut commands: &mut Commands,
    tool: &mut RoadTool,
    grid: &mut Grid,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    if adjacent_road_segment.area.min.position.x < tool.drag_area.min.position.x {
        spawn_road_segment(
            adjacent_road_segment.orientation,
            adjacent_road_segment.width,
            GridArea {
                min: adjacent_road_segment.area.min,
                max: GridCell::new(
                    tool.drag_area.adjacent_left().min.position.x,
                    adjacent_road_segment.area.max.position.y,
                ),
            },
            grid,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }

    if adjacent_road_segment.area.max.position.x > tool.drag_area.max.position.x {
        spawn_road_segment(
            adjacent_road_segment.orientation,
            adjacent_road_segment.width,
            GridArea {
                min: GridCell::new(
                    tool.drag_area.adjacent_right().max.position.x,
                    adjacent_road_segment.area.min.position.y,
                ),
                max: adjacent_road_segment.area.max,
            },
            grid,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }

    spawn_intersection(
        GridArea {
            min: GridCell::new(tool.drag_area.min.position.x, adjacent_road_segment.area.min.position.y),
            max: GridCell::new(tool.drag_area.max.position.x, adjacent_road_segment.area.max.position.y),
        },
        grid,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}
