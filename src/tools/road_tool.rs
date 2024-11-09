use crate::{
    graph::road_graph_events::*,
    graphics::camera::*,
    grid::{grid::*, grid_area::*, grid_cell::*, orientation::*},
    schedule::UpdateStage,
    tools::{road_events::*, toolbar::ToolState},
    types::{intersection::*, road_segment::*},
    ui::egui::MouseOver,
};
use bevy::{
    math::Affine2,
    prelude::*,
    render::texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
};
use std::f32::consts::FRAC_PI_2;

pub const ROAD_HEIGHT: f32 = 0.05;
pub const ROAD_TEXTURE_STRETCH: f32 = 5.0;

pub struct RoadToolPlugin;

impl Plugin for RoadToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tool)
            .add_event::<RequestRoad>()
            .add_event::<RequestIntersection>()
            .add_event::<RequestRoadSplit>()
            .add_event::<RequestRoadExtend>()
            .add_event::<RequestRoadBridge>()
            .add_systems(
                Update,
                (
                    (update_ground_position).in_set(UpdateStage::UpdateView).run_if(in_state(MouseOver::World)),
                    (adjust_tool_size, change_orientation, handle_action)
                        .in_set(UpdateStage::UserInput)
                        .run_if(in_state(MouseOver::World)),
                    (split_roads, extend_roads, bridge_roads).in_set(UpdateStage::HighLevelSideEffects),
                    (spawn_roads, spawn_intersections).in_set(UpdateStage::Spawning),
                )
                    .run_if(in_state(ToolState::Road)),
            );
    }
}

#[derive(Component, Debug)]
pub struct RoadTool {
    width: i32,
    ground_position: Vec3,
    drag_start_ground_position: Vec3,
    dragging: bool,
    drag_area: GridArea,
    orientation: GAxis,
}

impl RoadTool {
    fn new() -> Self {
        Self {
            width: 2,
            ground_position: Vec3::ZERO,
            drag_start_ground_position: Vec3::ZERO,
            dragging: false,
            drag_area: GridArea::at(Vec3::ZERO, 0, 0),
            orientation: GAxis::Z,
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
        if self.orientation == GAxis::Z {
            GridArea::at(self.drag_start_ground_position, self.width, 1)
        } else {
            GridArea::at(self.drag_start_ground_position, 1, self.width)
        }
    }

    fn drag_end_area(&self) -> GridArea {
        if self.orientation == GAxis::Z {
            GridArea::at(self.ground_position.with_x(self.drag_start_ground_position.x), self.width, 1)
        } else {
            GridArea::at(self.ground_position.with_z(self.drag_start_ground_position.z), 1, self.width)
        }
    }

    fn hover_area(&self) -> GridArea {
        if self.orientation == GAxis::Z {
            GridArea::at(self.ground_position, self.width, 1)
        } else {
            GridArea::at(self.ground_position, 1, self.width)
        }
    }

    fn drag_start_attach_area(&self) -> GridArea {
        let start = self.drag_start_area();
        let end = self.drag_end_area();

        if self.orientation == GAxis::Z {
            if end.max.pos.y >= start.max.pos.y {
                start.adjacent_bottom()
            } else {
                start.adjacent_top()
            }
        } else {
            if end.max.pos.x >= start.max.pos.x {
                start.adjacent_left()
            } else {
                start.adjacent_right()
            }
        }
    }

    fn drag_end_attach_area(&self) -> GridArea {
        let start = self.drag_start_area();
        let end = self.drag_end_area();

        if self.orientation == GAxis::Z {
            if end.max.pos.y >= start.max.pos.y {
                end.adjacent_top()
            } else {
                end.adjacent_bottom()
            }
        } else {
            if end.max.pos.x >= start.max.pos.x {
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

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    if let Some(distance) = ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up())) {
        let point = ray.get_point(distance);
        tool.ground_position = point;

        let area = tool.area();

        if tool.dragging {
            tool.drag_area = area;
        }

        let mut gizmo_color = if grid_query.single().is_valid_paint_area(area) {
            Color::linear_rgba(0.5, 0.0, 0.85, 0.8)
        } else {
            Color::linear_rgba(1.0, 0.0, 0.0, 0.25)
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

fn adjust_tool_size(mut query: Query<&mut RoadTool>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut tool = query.single_mut();

    if keyboard.just_pressed(KeyCode::KeyR) {
        tool.width += 2;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        tool.width -= 2;
    }

    tool.width = tool.width.max(2);
}

fn change_orientation(mut query: Query<&mut RoadTool>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut tool = query.single_mut();

    if keyboard.just_pressed(KeyCode::Tab) {
        tool.orientation = match tool.orientation {
            GAxis::X => GAxis::Z,
            GAxis::Z => GAxis::X,
        }
    }
}

fn handle_action(
    mut query: Query<&mut RoadTool>,
    mut grid_query: Query<&mut Grid>,
    segment_query: Query<&mut RoadSegment>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    creator: EventWriter<RequestRoad>,
    splitter: EventWriter<RequestRoadSplit>,
    extender: EventWriter<RequestRoadExtend>,
    intersector: EventWriter<RequestIntersection>,
    bridge: EventWriter<RequestRoadBridge>,
) {
    let mut tool = query.single_mut();
    let mut grid = grid_query.single_mut();

    if mouse.just_pressed(MouseButton::Left) && !keyboard.any_pressed([KeyCode::AltLeft, KeyCode::ControlLeft]) {
        if !tool.dragging {
            tool.dragging = true;
            tool.drag_start_ground_position = tool.ground_position;
        } else {
            handle_end_drag(
                &mut tool,
                &mut grid,
                segment_query,
                creator,
                splitter,
                extender,
                intersector,
                bridge,
            );
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
    mut creator: EventWriter<RequestRoad>,
    mut splitter: EventWriter<RequestRoadSplit>,
    mut extender: EventWriter<RequestRoadExtend>,
    mut intersector: EventWriter<RequestIntersection>,
    mut bridge: EventWriter<RequestRoadBridge>,
) {
    if grid.is_valid_paint_area(tool.drag_area) {
        let mut extend_start = false;
        let mut extend_end = false;
        let mut extend_entities = Vec::<Entity>::new();

        if let Some(adjacent_entity) = grid.single_entity_in_area(tool.drag_start_attach_area()) {
            if let Ok(adj) = segment_query.get(adjacent_entity) {
                if adj.orientation != tool.orientation {
                    let intersection_area = adj.get_intersection_area(tool.drag_area);
                    splitter.send(RequestRoadSplit::new(adjacent_entity, intersection_area));
                    intersector.send(RequestIntersection::new(intersection_area));
                } else if adj.drive_width() == tool.width {
                    extend_start = true;
                    extend_entities.push(adjacent_entity);
                }
            }
        }

        if let Some(adjacent_entity) = grid.single_entity_in_area(tool.drag_end_attach_area()) {
            if let Ok(adj) = segment_query.get(adjacent_entity) {
                if adj.orientation != tool.orientation {
                    let intersection_area = adj.get_intersection_area(tool.drag_area);
                    splitter.send(RequestRoadSplit::new(adjacent_entity, intersection_area));
                    intersector.send(RequestIntersection::new(intersection_area));
                } else if adj.drive_width() == tool.width {
                    extend_end = true;
                    extend_entities.push(adjacent_entity);
                }
            }
        }

        if !extend_start && !extend_end {
            creator.send(RequestRoad::new(tool.drag_area, tool.orientation));
        } else if extend_start && extend_end {
            bridge.send(RequestRoadBridge::new(extend_entities[0], extend_entities[1]));
        } else {
            for adjacent_entity in extend_entities {
                extender.send(RequestRoadExtend::new(adjacent_entity, tool.drag_area));
            }
        }
    }

    tool.dragging = false;
}

fn spawn_roads(
    mut spawner: EventReader<RequestRoad>,
    mut event: EventWriter<OnRoadSpawned>,
    mut commands: Commands,
    mut grid_query: Query<&mut Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut grid = grid_query.single_mut();

    for &RequestRoad { area, orientation } in spawner.read() {
        let width = match orientation {
            GAxis::Z => area.cell_dimensions().x,
            GAxis::X => area.cell_dimensions().y,
        };

        let length = match orientation {
            GAxis::Z => area.cell_dimensions().y,
            GAxis::X => area.cell_dimensions().x,
        };

        let texture = match width {
            6 => "textures/three_lanes.png",
            4 => "textures/two_lanes.png",
            _ => "textures/one_lane.png",
        };

        let material = StandardMaterial {
            base_color_texture: Some(asset_server.load_with_settings(texture, |s: &mut _| {
                *s = ImageLoaderSettings {
                    sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        ..default()
                    }),
                    ..default()
                }
            })),
            uv_transform: Affine2::from_scale(Vec2::new(length as f32 / ROAD_TEXTURE_STRETCH, 1.0)),
            ..default()
        };

        let model = PbrBundle {
            mesh: meshes.add(match orientation {
                GAxis::Z => Cuboid::new(area.dimensions().y, ROAD_HEIGHT, area.dimensions().x),
                GAxis::X => Cuboid::new(area.dimensions().x, ROAD_HEIGHT, area.dimensions().y),
            }),
            material: materials.add(material),
            transform: Transform::from_translation(area.center().with_y(ROAD_HEIGHT / 2.0)).with_rotation(
                match orientation {
                    GAxis::Z => Quat::from_rotation_y(std::f32::consts::PI / 2.0),
                    GAxis::X => Quat::IDENTITY,
                },
            ),
            ..default()
        };

        let entity = commands.spawn((model, RoadSegment::new(area, orientation))).id();
        grid.mark_area_occupied(area, entity);
        event.send(OnRoadSpawned(entity));
    }
}

fn spawn_intersections(
    mut spawner: EventReader<RequestIntersection>,
    mut event: EventWriter<OnIntersectionSpawned>,
    mut commands: Commands,
    mut grid_query: Query<&mut Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for &RequestIntersection { area } in spawner.read() {
        let model = PbrBundle {
            mesh: meshes.add(Cuboid::new(area.dimensions().x, ROAD_HEIGHT, area.dimensions().y)),
            material: materials.add(asset_server.load("textures/intersection.png")),
            transform: Transform::from_translation(area.center().with_y(ROAD_HEIGHT / 2.0)),
            ..default()
        };

        let entity = commands.spawn((model, Intersection::new(area))).id();
        grid_query.single_mut().mark_area_occupied(area, entity);
        event.send(OnIntersectionSpawned(entity));
    }
}

fn split_roads(
    mut split_event: EventReader<RequestRoadSplit>,
    mut destroyer: EventWriter<OnRoadDestroyed>,
    segment_query: Query<&mut RoadSegment>,
    mut roads: EventWriter<RequestRoad>,
) {
    for &RequestRoadSplit { entity, split_area } in split_event.read() {
        if let Ok(segment) = segment_query.get(entity) {
            if segment.orientation == GAxis::Z {
                if segment.area.min.pos.y < split_area.min.pos.y {
                    let split_max = GridCell::new(segment.area.max.pos.x, split_area.adjacent_bottom().min.pos.y);
                    let road_area = GridArea::new(segment.area.min, split_max);
                    roads.send(RequestRoad::new(road_area, segment.orientation));
                }

                if segment.area.max.pos.y > split_area.max.pos.y {
                    let split_min = GridCell::new(segment.area.min.pos.x, split_area.adjacent_top().max.pos.y);
                    let road_area = GridArea::new(split_min, segment.area.max);
                    roads.send(RequestRoad::new(road_area, segment.orientation));
                }
            } else {
                if segment.area.min.pos.x < split_area.min.pos.x {
                    let split_max = GridCell::new(split_area.adjacent_left().min.pos.x, segment.area.max.pos.y);
                    let road_area = GridArea::new(segment.area.min, split_max);
                    roads.send(RequestRoad::new(road_area, segment.orientation));
                }

                if segment.area.max.pos.x > split_area.max.pos.x {
                    let split_min = GridCell::new(split_area.adjacent_right().max.pos.x, segment.area.min.pos.y);
                    let road_area = GridArea::new(split_min, segment.area.max);
                    roads.send(RequestRoad::new(road_area, segment.orientation));
                }
            }

            destroyer.send(OnRoadDestroyed(entity));
        }
    }
}

fn extend_roads(
    mut extend_event: EventReader<RequestRoadExtend>,
    mut destroyer: EventWriter<OnRoadDestroyed>,
    segment_query: Query<&mut RoadSegment>,
    mut roads: EventWriter<RequestRoad>,
) {
    for &RequestRoadExtend { entity, extension } in extend_event.read() {
        if let Ok(original_segment) = segment_query.get(entity) {
            let extended_area = original_segment.area.union(extension);
            roads.send(RequestRoad::new(extended_area, original_segment.orientation));
            destroyer.send(OnRoadDestroyed(entity));
        }
    }
}

fn bridge_roads(
    mut bridge_event: EventReader<RequestRoadBridge>,
    mut destroyer: EventWriter<OnRoadDestroyed>,
    segment_query: Query<&mut RoadSegment>,
    mut roads: EventWriter<RequestRoad>,
) {
    for &RequestRoadBridge { first, second } in bridge_event.read() {
        if let Ok(first_segment) = segment_query.get(first) {
            if let Ok(second_segment) = segment_query.get(second) {
                let extended_area = first_segment.area.union(second_segment.area);
                roads.send(RequestRoad::new(extended_area, first_segment.orientation));
                destroyer.send(OnRoadDestroyed(first));
                destroyer.send(OnRoadDestroyed(second));
            }
        }
    }
}
