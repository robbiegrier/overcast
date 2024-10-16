use crate::{
    camera::PlayerCameraController,
    graph_events::*,
    grid::{Grid, Ground},
    grid_area::GridArea,
    grid_cell::GridCell,
    road_events::*,
    road_segment::RoadSegment,
    toolbar::ToolState,
};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

const ROAD_HEIGHT: f32 = 0.05;

pub struct RoadToolPlugin;

impl Plugin for RoadToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tool)
            .add_event::<RoadCreateEvent>()
            .add_event::<IntersectionCreateEvent>()
            .add_event::<RoadSplitEvent>()
            .add_event::<RoadExtendEvent>()
            .add_systems(
                Update,
                (
                    update_ground_position,
                    (adjust_tool_size, change_orientation, handle_action)
                        .after(update_ground_position)
                        .before(split_roads)
                        .before(spawn_roads)
                        .before(spawn_intersections),
                    (split_roads, extend_roads).before(spawn_roads).before(spawn_intersections),
                    spawn_roads,
                    spawn_intersections,
                )
                    .run_if(in_state(ToolState::Road)),
            );
    }
}

#[derive(Component, Debug)]
pub struct Intersection {
    pub area: GridArea,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum RoadToolMode {
    Spawner,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Axis {
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
    orientation: Axis,
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
            orientation: Axis::Z,
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
        if self.orientation == Axis::Z {
            GridArea::at(self.drag_start_ground_position, self.width, 1)
        } else {
            GridArea::at(self.drag_start_ground_position, 1, self.width)
        }
    }

    fn drag_end_area(&self) -> GridArea {
        if self.orientation == Axis::Z {
            GridArea::at(self.ground_position.with_x(self.drag_start_ground_position.x), self.width, 1)
        } else {
            GridArea::at(self.ground_position.with_z(self.drag_start_ground_position.z), 1, self.width)
        }
    }

    fn hover_area(&self) -> GridArea {
        if self.orientation == Axis::Z {
            GridArea::at(self.ground_position, self.width, 1)
        } else {
            GridArea::at(self.ground_position, 1, self.width)
        }
    }

    fn drag_start_attach_area(&self) -> GridArea {
        let start = self.drag_start_area();
        let end = self.drag_end_area();

        if self.orientation == Axis::Z {
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

        if self.orientation == Axis::Z {
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
            Axis::X => Axis::Z,
            Axis::Z => Axis::X,
        }
    }
}

fn handle_action(
    mut query: Query<&mut RoadTool>,
    mut grid_query: Query<&mut Grid>,
    segment_query: Query<&mut RoadSegment>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    creator: EventWriter<RoadCreateEvent>,
    splitter: EventWriter<RoadSplitEvent>,
    extender: EventWriter<RoadExtendEvent>,
    intersector: EventWriter<IntersectionCreateEvent>,
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
                    handle_end_drag(&mut tool, &mut grid, segment_query, creator, splitter, extender, intersector);
                }
            }
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
    mut creator: EventWriter<RoadCreateEvent>,
    mut splitter: EventWriter<RoadSplitEvent>,
    mut extender: EventWriter<RoadExtendEvent>,
    mut intersector: EventWriter<IntersectionCreateEvent>,
) {
    if grid.is_valid_paint_area(tool.drag_area) {
        let mut add_road = true;

        if let Some(adjacent_entity) = grid.single_entity_in_area(tool.drag_start_attach_area()) {
            if let Ok(adj) = segment_query.get(adjacent_entity) {
                if adj.orientation != tool.orientation {
                    // println!("at start, create intersection");
                    let intersection_area = adj.get_intersection_area(tool.drag_area);
                    splitter.send(RoadSplitEvent::new(adjacent_entity, intersection_area));
                    intersector.send(IntersectionCreateEvent::new(intersection_area));
                } else if adj.drive_width() == tool.width {
                    // println!("at start, create extension");
                    extender.send(RoadExtendEvent::new(adjacent_entity, tool.drag_area));
                    add_road = false;
                }
            }
        }

        if let Some(adjacent_entity) = grid.single_entity_in_area(tool.drag_end_attach_area()) {
            if let Ok(adj) = segment_query.get(adjacent_entity) {
                if adj.orientation != tool.orientation {
                    // println!("at end, create intersection");
                    let intersection_area = adj.get_intersection_area(tool.drag_area);
                    splitter.send(RoadSplitEvent::new(adjacent_entity, intersection_area));
                    intersector.send(IntersectionCreateEvent::new(intersection_area));
                } else if adj.drive_width() == tool.width {
                    // println!("at end, create extension");
                    extender.send(RoadExtendEvent::new(adjacent_entity, tool.drag_area));
                    add_road = false;
                }
            }
        }

        if add_road {
            creator.send(RoadCreateEvent::new(tool.drag_area, tool.orientation));
        }
    }

    tool.dragging = false;
}

fn spawn_roads(
    mut road_create_event_reader: EventReader<RoadCreateEvent>,
    mut graph_event: EventWriter<GraphEdgeAddEvent>,
    mut commands: Commands,
    mut grid_query: Query<&mut Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for &RoadCreateEvent { area, orientation } in road_create_event_reader.read() {
        let size = area.dimensions();
        let road_color = if orientation == Axis::Z { 0.05 } else { 0.1 };

        let entity = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(size.x, ROAD_HEIGHT, size.y)),
                    material: materials.add(Color::linear_rgb(road_color, road_color, road_color)),
                    transform: Transform::from_translation(area.center().with_y(ROAD_HEIGHT / 2.0)),
                    ..default()
                },
                RoadSegment::new(area, orientation),
            ))
            .id();

        grid_query.single_mut().mark_area_occupied(area, entity);
        graph_event.send(GraphEdgeAddEvent(entity));
    }
}

fn spawn_intersections(
    mut intersection_event: EventReader<IntersectionCreateEvent>,
    mut graph_event: EventWriter<GraphNodeAddEvent>,
    mut commands: Commands,
    mut grid_query: Query<&mut Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for &IntersectionCreateEvent { area } in intersection_event.read() {
        let size = area.dimensions();

        let entity = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(size.x, ROAD_HEIGHT, size.y)),
                    material: materials.add(Color::linear_rgb(0.0, 0.1, 0.3)),
                    transform: Transform::from_translation(area.center().with_y(ROAD_HEIGHT / 2.0)),
                    ..default()
                },
                Intersection { area },
            ))
            .id();

        grid_query.single_mut().mark_area_occupied(area, entity);
        graph_event.send(GraphNodeAddEvent(entity));
    }
}

fn split_roads(
    mut split_event: EventReader<RoadSplitEvent>,
    mut rem_event: EventWriter<GraphEdgeRemoveEvent>,
    segment_query: Query<&mut RoadSegment>,
    mut roads: EventWriter<RoadCreateEvent>,
    mut grid_query: Query<&mut Grid>,
    mut commands: Commands,
) {
    for &RoadSplitEvent { entity, split_area } in split_event.read() {
        if let Ok(segment) = segment_query.get(entity) {
            grid_query.single_mut().erase(entity);

            if segment.orientation == Axis::Z {
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
            rem_event.send(GraphEdgeRemoveEvent(entity));
        }
    }
}

fn extend_roads(
    mut extend_event: EventReader<RoadExtendEvent>,
    mut rem_event: EventWriter<GraphEdgeRemoveEvent>,
    segment_query: Query<&mut RoadSegment>,
    mut roads: EventWriter<RoadCreateEvent>,
    mut commands: Commands,
) {
    for &RoadExtendEvent { entity, extension } in extend_event.read() {
        if let Ok(original_segment) = segment_query.get(entity) {
            let extended_area = original_segment.area.union(extension);
            roads.send(RoadCreateEvent::new(extended_area, original_segment.orientation));
            commands.entity(entity).despawn();
            rem_event.send(GraphEdgeRemoveEvent(entity));
        }
    }
}
