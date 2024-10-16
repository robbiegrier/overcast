use crate::{
    camera::PlayerCameraController,
    grid::{Grid, Ground},
    grid_area::GridArea,
    grid_cell::GridCell,
    road_events::*,
    toolbar::ToolState,
};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub struct RoadToolPlugin;

impl Plugin for RoadToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tool)
            .add_event::<RoadCreateEvent>()
            .add_event::<IntersectionCreateEvent>()
            .add_event::<RoadSplitEvent>()
            .add_systems(
                Update,
                (
                    update_ground_position,
                    split_roads.before(spawn_roads).before(spawn_intersections),
                    (adjust_tool_size, change_orientation, handle_action)
                        .after(update_ground_position)
                        .before(spawn_roads)
                        .before(spawn_intersections)
                        .before(split_roads),
                    spawn_roads,
                    spawn_intersections,
                )
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
    mut query: Query<&mut RoadTool>,
    mut grid_query: Query<&mut Grid>,
    segment_query: Query<&mut RoadSegment>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    event: EventWriter<RoadCreateEvent>,
    splitter: EventWriter<RoadSplitEvent>,
    intersection_event: EventWriter<IntersectionCreateEvent>,
) {
    let mut tool = query.single_mut();
    let mut grid = grid_query.single_mut();

    if mouse.just_pressed(MouseButton::Left) && !keyboard.any_pressed([KeyCode::AltLeft, KeyCode::ControlLeft]) {
        match tool.mode {
            RoadToolMode::Spawner => {
                if !tool.dragging {
                    tool.dragging = true;
                    tool.drag_start_ground_position = tool.ground_position;
                } else {
                    handle_end_drag(&mut tool, &mut grid, segment_query, event, splitter, intersection_event);
                }
            }
            RoadToolMode::Eraser => {}
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        tool.dragging = false;
    }
}

fn handle_end_drag(
    tool: &mut RoadTool,
    grid: &mut Grid,
    segment_query: Query<&mut RoadSegment>,
    mut event: EventWriter<RoadCreateEvent>,
    mut splitter: EventWriter<RoadSplitEvent>,
    mut intersection_event: EventWriter<IntersectionCreateEvent>,
) {
    if grid.is_valid_paint_area(tool.drag_area) {
        if let Some(adjacent_entity) = grid.single_entity_in_area(tool.drag_start_attach_area()) {
            if let Ok(adj) = segment_query.get(adjacent_entity) {
                if adj.orientation != tool.orientation {
                    println!("at start, create intersection");
                    if adj.orientation == RoadOrientation::Z {
                        let intersection_area = GridArea::new(
                            GridCell::new(adj.area.min.position.x, tool.drag_area.min.position.y),
                            GridCell::new(adj.area.max.position.x, tool.drag_area.max.position.y),
                        );
                        splitter.send(RoadSplitEvent::new(adjacent_entity, intersection_area));
                        intersection_event.send(IntersectionCreateEvent::new(intersection_area));
                    } else {
                        let intersection_area = GridArea::new(
                            GridCell::new(tool.drag_area.min.position.x, adj.area.min.position.y),
                            GridCell::new(tool.drag_area.max.position.x, adj.area.max.position.y),
                        );
                        splitter.send(RoadSplitEvent::new(adjacent_entity, intersection_area));
                        intersection_event.send(IntersectionCreateEvent::new(intersection_area));
                    }
                } else if adj.width == tool.width {
                    println!("at start, create extension");
                }
            }
        }

        if let Some(adjacent_entity) = grid.single_entity_in_area(tool.drag_end_attach_area()) {
            if let Ok(adj) = segment_query.get(adjacent_entity) {
                if adj.orientation != tool.orientation {
                    println!("at end, create intersection");
                    if adj.orientation == RoadOrientation::Z {
                        let intersection_area = GridArea::new(
                            GridCell::new(adj.area.min.position.x, tool.drag_area.min.position.y),
                            GridCell::new(adj.area.max.position.x, tool.drag_area.max.position.y),
                        );
                        splitter.send(RoadSplitEvent::new(adjacent_entity, intersection_area));
                        intersection_event.send(IntersectionCreateEvent::new(intersection_area));
                    } else {
                        let intersection_area = GridArea::new(
                            GridCell::new(tool.drag_area.min.position.x, adj.area.min.position.y),
                            GridCell::new(tool.drag_area.max.position.x, adj.area.max.position.y),
                        );
                        splitter.send(RoadSplitEvent::new(adjacent_entity, intersection_area));
                        intersection_event.send(IntersectionCreateEvent::new(intersection_area));
                    }
                } else if adj.width == tool.width {
                    println!("at end, create extension");
                }
            }
        }

        event.send(RoadCreateEvent::new(tool.drag_area, tool.orientation));
    }

    tool.dragging = false;
}

fn spawn_roads(
    mut road_create_event_reader: EventReader<RoadCreateEvent>,
    mut commands: Commands,
    mut grid_query: Query<&mut Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for &RoadCreateEvent { area, orientation } in road_create_event_reader.read() {
        let size = area.dimensions();
        let road_height = 0.05;
        let road_color = if orientation == RoadOrientation::Z { 0.05 } else { 0.1 };
        let width = if orientation == RoadOrientation::Z {
            area.cell_dimenions().x
        } else {
            area.cell_dimenions().y
        };

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

        grid_query.single_mut().mark_area_occupied(area, entity);
    }
}

fn spawn_intersections(
    mut intersection_event: EventReader<IntersectionCreateEvent>,
    mut commands: Commands,
    mut grid_query: Query<&mut Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for &IntersectionCreateEvent { area } in intersection_event.read() {
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
                Intersection { area },
            ))
            .id();

        grid_query.single_mut().mark_area_occupied(area, entity);
    }
}

fn split_roads(
    mut split_event: EventReader<RoadSplitEvent>,
    segment_query: Query<&mut RoadSegment>,
    mut roads: EventWriter<RoadCreateEvent>,
    mut grid_query: Query<&mut Grid>,
    mut commands: Commands,
) {
    for &RoadSplitEvent { entity, split_area } in split_event.read() {
        if let Ok(segment) = segment_query.get(entity) {
            grid_query.single_mut().erase(entity);

            if segment.orientation == RoadOrientation::Z {
                if segment.area.min.position.y < split_area.min.position.y {
                    let split_max = GridCell::new(segment.area.max.position.x, split_area.adjacent_bottom().min.position.y);
                    let road_area = GridArea::new(segment.area.min, split_max);
                    roads.send(RoadCreateEvent::new(road_area, segment.orientation));
                }

                if segment.area.max.position.y > split_area.max.position.y {
                    let split_min = GridCell::new(segment.area.min.position.x, split_area.adjacent_top().max.position.y);
                    let road_area = GridArea::new(split_min, segment.area.max);
                    roads.send(RoadCreateEvent::new(road_area, segment.orientation));
                }
            } else {
                if segment.area.min.position.x < split_area.min.position.x {
                    let split_max = GridCell::new(split_area.adjacent_left().min.position.x, segment.area.max.position.y);
                    let road_area = GridArea::new(segment.area.min, split_max);
                    roads.send(RoadCreateEvent::new(road_area, segment.orientation));
                }

                if segment.area.max.position.x > split_area.max.position.x {
                    let split_min = GridCell::new(split_area.adjacent_right().max.position.x, segment.area.min.position.y);
                    let road_area = GridArea::new(split_min, segment.area.max);
                    roads.send(RoadCreateEvent::new(road_area, segment.orientation));
                }
            }

            commands.entity(entity).despawn();
        }
    }
}
