use crate::{
    grid::{grid_area::GridArea, orientation::*},
    schedule::UpdateStage,
    tools::road_tool::ROAD_HEIGHT,
    types::{building::*, intersection::*, road_segment::*},
};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_mod_raycast::prelude::*;
use rand::{seq::IteratorRandom, Rng};

const VEHICLE_HEIGHT: f32 = 0.25;
const VEHICLE_LENGTH: f32 = VEHICLE_HEIGHT * 2.0;
const VEHICLE_MAX_SPEED: f32 = 1.5;
const MAX_SPEED_VARIATION: f32 = 0.5;
const SPAWN_TIME_SECONDS: f32 = 0.25;
const INTERSECTION_OFFSET: f32 = 0.2;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AiVisualizationState {
    Visualize,
    #[default]
    Hide,
}

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DeferredRaycastingPlugin::<VehicleRaycastSet>::default())
            .insert_resource(RaycastPluginState::<VehicleRaycastSet>::default())
            .init_state::<AiVisualizationState>()
            .add_event::<RequestVehicleSpawn>()
            .insert_resource(SpawnTimer {
                timer: Timer::from_seconds(SPAWN_TIME_SECONDS, TimerMode::Repeating),
            })
            .add_systems(
                Update,
                (
                    (toggle_ai_vizualization, spawn_vehicle_on_key_press, spawn_vehicle_on_timer)
                        .in_set(UpdateStage::UserInput),
                    (spawn_vehicle).in_set(UpdateStage::Spawning),
                    (update_vehicles, update_speed, execute_movement, execute_turning).in_set(UpdateStage::AiBehavior),
                    (visualize_path, visualize_vehicle_ai)
                        .in_set(UpdateStage::Visualize)
                        .run_if(in_state(AiVisualizationState::Visualize)),
                ),
            );
    }
}

#[derive(Reflect)]
struct VehicleRaycastSet;

#[derive(Component, Debug)]
pub struct Vehicle {
    pub path: Vec<Entity>,
    pub path_index: usize,
    pub speed: f32,
    pub max_speed: f32,
    pub follow: Vec3,
    pub checkpoint: Vec3,
    pub lane: i32,
}

impl Vehicle {
    fn new(path: Vec<Entity>, max_speed: f32) -> Self {
        Self {
            path,
            path_index: 0,
            speed: 0.0,
            max_speed,
            follow: Vec3::ZERO,
            checkpoint: Vec3::ZERO,
            lane: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StepType {
    Road,
    Intersection,
    Building,
}

fn get_step_type(step_entity: Entity, dest_query: &Query<&Building>, edge_query: &Query<&RoadSegment>) -> StepType {
    if edge_query.contains(step_entity) {
        StepType::Road
    } else if dest_query.contains(step_entity) {
        StepType::Building
    } else {
        StepType::Intersection
    }
}

fn direction_to_area(segment: &RoadSegment, area: GridArea) -> GDir {
    match segment.orientation {
        GAxis::Z => {
            if area.center().z > segment.area.center().z {
                GDir::North
            } else {
                GDir::South
            }
        }
        GAxis::X => {
            if area.center().x > segment.area.center().x {
                GDir::West
            } else {
                GDir::East
            }
        }
    }
}

fn direction_to_building(segment: &RoadSegment, building: &Building, pos: Vec3) -> GDir {
    match segment.orientation {
        GAxis::Z => {
            if building.area.center().z > pos.z {
                GDir::North
            } else {
                GDir::South
            }
        }
        GAxis::X => {
            if building.area.center().x > pos.x {
                GDir::West
            } else {
                GDir::East
            }
        }
    }
}

fn get_intersection_goal(intersection: &Intersection, direction: GDir, start_pos: Vec3) -> Vec3 {
    match direction {
        GDir::North => intersection.area.center().with_x(start_pos.x).with_y(start_pos.y),
        GDir::South => intersection.area.center().with_x(start_pos.x).with_y(start_pos.y),
        GDir::East => intersection.area.center().with_z(start_pos.z).with_y(start_pos.y),
        GDir::West => intersection.area.center().with_z(start_pos.z).with_y(start_pos.y),
    }
}

fn get_lane_for_turn(curr: &RoadSegment, next: &RoadSegment, prev: i32) -> i32 {
    let z_less = next.area().center().z < curr.area().center().z;
    let x_less = next.area().center().x < curr.area().center().x;
    if curr.orientation == next.orientation {
        prev
    } else if next.orientation == GAxis::X {
        match z_less {
            true => match x_less {
                true => curr.num_lanes() - 1,
                false => 0,
            },
            false => match x_less {
                false => curr.num_lanes() - 1,
                true => 0,
            },
        }
    } else {
        match x_less {
            true => match z_less {
                false => curr.num_lanes() - 1,
                true => 0,
            },
            false => match z_less {
                true => curr.num_lanes() - 1,
                false => 0,
            },
        }
    }
}

fn execute_turning(mut vehicle_query: Query<(&Vehicle, &mut Transform)>, time: Res<Time>) {
    for (vehicle, mut transform) in &mut vehicle_query {
        let follow_vec = vehicle.follow.with_y(0.0) - transform.translation.with_y(0.0);
        let follow_dir = follow_vec.normalize();
        let dot = follow_dir.dot(transform.left().as_vec3());

        if dot > 0.01 {
            let scalar = follow_dir.angle_between(transform.right().as_vec3());
            transform.rotate_y(1.0 * scalar * time.delta_seconds());
        } else if dot < -0.01 {
            let scalar = follow_dir.angle_between(transform.left().as_vec3());
            transform.rotate_y(-1.0 * scalar * time.delta_seconds());
        } else {
            transform.look_at(vehicle.follow, Vec3::new(0.0, 1.0, 0.0));
        }
    }
}

fn update_speed(mut vehicle_query: Query<(&mut Vehicle, &RaycastSource<VehicleRaycastSet>)>, time: Res<Time>) {
    for (mut vehicle, raycast) in &mut vehicle_query {
        vehicle.speed = vehicle.speed.lerp(vehicle.max_speed, time.delta_seconds() * 0.5);

        let slow_dist = 2.0;
        let stop_dist = 1.0;
        if let Some((_, hit)) = raycast.get_nearest_intersection() {
            if hit.distance() < slow_dist {
                vehicle.speed -= (slow_dist - hit.distance()).max(0.0) * time.delta_seconds();
                vehicle.speed = vehicle.speed.max(0.0);
            }

            if hit.distance() < stop_dist {
                vehicle.speed = 0.0;
            }
        }
    }
}

fn execute_movement(mut vehicle_query: Query<(&Vehicle, &mut Transform)>, time: Res<Time>) {
    for (vehicle, mut transform) in &mut vehicle_query {
        let translate_dir = transform.forward().as_vec3();
        transform.translation += vehicle.speed * translate_dir * time.delta_seconds();
    }
}

fn toggle_ai_vizualization(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AiVisualizationState>>,
    state: Res<State<AiVisualizationState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        next_state.set({
            match state.get() {
                AiVisualizationState::Hide => AiVisualizationState::Visualize,
                AiVisualizationState::Visualize => AiVisualizationState::Hide,
            }
        });
    }
}

fn visualize_vehicle_ai(mut vehicle_query: Query<(&Vehicle, &Transform)>, mut gizmos: Gizmos) {
    for (vehicle, transform) in &mut vehicle_query {
        gizmos.line(transform.translation, vehicle.checkpoint, Color::linear_rgb(1.0, 1.0, 0.0));
        gizmos.arrow(transform.translation, vehicle.follow, Color::linear_rgb(0.0, 1.0, 0.0));
    }
}

fn update_vehicles(
    mut commands: Commands,
    mut vehicle_query: Query<(Entity, &mut Vehicle, &mut Transform)>,
    segment_query: Query<&RoadSegment>,
    intersection_query: Query<&Intersection>,
    building_query: Query<&Building>,
) {
    for (entity, mut vehicle, mut transform) in &mut vehicle_query {
        if vehicle.path_index >= vehicle.path.len() - 1 {
            commands.entity(entity).despawn_recursive();
            return;
        }

        let curr = vehicle.path[vehicle.path_index];
        let next = vehicle.path[vehicle.path_index + 1];

        let curr_type = get_step_type(curr, &building_query, &segment_query);
        let next_type = get_step_type(next, &building_query, &segment_query);

        vehicle.checkpoint = transform.translation;
        vehicle.follow = transform.translation;

        if curr_type == StepType::Building && next_type == StepType::Road {
            if let Ok(segment) = segment_query.get(next) {
                let lane_pos = segment.get_lane_pos(transform.translation);
                transform.translation = lane_pos;
                vehicle.path_index += 1;
                return;
            }
        } else if curr_type == StepType::Road && next_type == StepType::Building {
            if let Ok(building) = building_query.get(next) {
                if let Ok(segment) = segment_query.get(curr) {
                    let approach_dir = direction_to_building(segment, building, transform.translation);
                    let target = building.area.center().with_y(transform.translation.y);
                    vehicle.checkpoint = segment.clamp_to_lane(approach_dir, 0, target);

                    let lane_pos = segment.clamp_to_lane(approach_dir, 0, transform.translation);
                    let current_vec = transform.translation - vehicle.checkpoint;
                    let desired_vec = lane_pos - vehicle.checkpoint;
                    let proj = vehicle.checkpoint + (current_vec).project_onto(desired_vec);
                    let interp_proj = proj + (vehicle.checkpoint - proj).normalize() * 0.5;
                    vehicle.follow = interp_proj;

                    if transform.translation.distance(vehicle.checkpoint) < 1.0 {
                        vehicle.path_index += 1;
                        return;
                    }
                }
            }
        } else if curr_type == StepType::Road && next_type == StepType::Intersection {
            if let Ok(intersection) = intersection_query.get(next) {
                if let Ok(segment) = segment_query.get(curr) {
                    let approach_dir = direction_to_area(segment, intersection.area());
                    vehicle.checkpoint = get_intersection_goal(intersection, approach_dir, transform.translation);

                    if let Ok(next_segment) = segment_query.get(vehicle.path[vehicle.path_index + 2]) {
                        vehicle.lane = get_lane_for_turn(segment, next_segment, vehicle.lane);
                    }

                    let lane_pos = segment.clamp_to_lane(approach_dir, vehicle.lane, transform.translation);
                    let current_vec = transform.translation - vehicle.checkpoint;
                    let desired_vec = lane_pos - vehicle.checkpoint;
                    let proj = vehicle.checkpoint + (current_vec).project_onto(desired_vec);
                    let interp_proj = proj + (vehicle.checkpoint - proj).normalize() * 0.5;
                    vehicle.follow = interp_proj;

                    if intersection.area.contains_point_3d(transform.translation) {
                        vehicle.path_index += 1;
                        return;
                    }
                }
            }
        } else if curr_type == StepType::Intersection {
            if let Ok(intersection) = intersection_query.get(curr) {
                if let Ok(next_segment) = segment_query.get(next) {
                    let approach_dir = direction_to_area(next_segment, intersection.area()).inverse();

                    if let Ok(prev_segment) = segment_query.get(vehicle.path[vehicle.path_index - 1]) {
                        vehicle.lane = get_lane_for_turn(prev_segment, next_segment, vehicle.lane);
                    }

                    vehicle.checkpoint = next_segment.clamp_to_lane(approach_dir, vehicle.lane, transform.translation);
                    vehicle.checkpoint += approach_dir.as_vec3() * INTERSECTION_OFFSET;

                    let interp_proj = transform.translation + (vehicle.checkpoint - transform.translation).normalize() * 0.5;
                    vehicle.follow = interp_proj;

                    if next_segment.area.contains_point_3d(transform.translation) {
                        vehicle.path_index += 1;
                        return;
                    }
                }
            }
        }
    }
}

#[derive(Event, Debug)]
pub struct RequestVehicleSpawn;

#[derive(Resource, Debug)]
pub struct SpawnTimer {
    timer: Timer,
}

fn spawn_vehicle_on_key_press(keyboard: Res<ButtonInput<KeyCode>>, mut request: EventWriter<RequestVehicleSpawn>) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        request.send(RequestVehicleSpawn);
    }
}

fn spawn_vehicle_on_timer(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut request: EventWriter<RequestVehicleSpawn>,
    time: Res<Time>,
    mut spawn_timer: ResMut<SpawnTimer>,
) {
    if keyboard.pressed(KeyCode::KeyL) {
        spawn_timer.timer.tick(time.delta());
        if spawn_timer.timer.just_finished() {
            request.send(RequestVehicleSpawn);
        }
    }
}

fn spawn_vehicle(
    building_query: Query<(Entity, &Building)>,
    segment_query: Query<(Entity, &RoadSegment)>,
    inter_query: Query<(Entity, &Intersection)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut request: EventReader<RequestVehicleSpawn>,
) {
    for _ in request.read() {
        let mut rng = rand::thread_rng();
        let choose = building_query.iter().choose_multiple(&mut rng, 2);

        if choose.len() < 2 {
            println!("not enough buildings to make a path");
            return;
        }

        let (start_entity, _start) = choose[0];
        let (end_entity, _end) = choose[1];

        let mut frontier = Vec::<Entity>::new();
        let mut visited = HashSet::<Entity>::new();
        let mut parent_map = HashMap::<Entity, Entity>::new();

        frontier.push(start_entity);

        let mut path_found = false;

        while let Some(curr) = frontier.pop() {
            visited.insert(curr);
            // if curr is destination
            if let Ok((e, dest)) = building_query.get(curr) {
                if e == end_entity {
                    path_found = true;
                    break;
                }

                if !dest.roads.is_empty() {
                    let start_road = dest.roads.iter().take(1).next().unwrap();
                    frontier.push(*start_road);
                    parent_map.insert(*start_road, curr);
                }
            }
            // if curr is edge
            else if let Ok((_e, edge)) = segment_query.get(curr) {
                // if end goal is a destination here, go to it
                if edge.dests.contains(&end_entity) {
                    frontier.push(end_entity);
                    parent_map.insert(end_entity, curr);
                }
                // Add endpoints of this edge
                else {
                    if let Some(endpoint0) = edge.ends[0] {
                        if let Ok((en0, _n0)) = inter_query.get(endpoint0) {
                            if !visited.contains(&en0) {
                                frontier.push(en0);
                                parent_map.insert(en0, curr);
                            }
                        }
                    }
                    if let Some(endpoint1) = edge.ends[1] {
                        if let Ok((en1, _n1)) = inter_query.get(endpoint1) {
                            if !visited.contains(&en1) {
                                frontier.push(en1);
                                parent_map.insert(en1, curr);
                            }
                        }
                    }
                }
            }
            // if curr is a node, add connected edges
            else if let Ok((_e, node)) = inter_query.get(curr) {
                for slot in &node.roads {
                    if let Some(road) = slot {
                        if !visited.contains(road) {
                            frontier.push(*road);
                            parent_map.insert(*road, curr);
                        }
                    }
                }
            }
        }

        if path_found {
            let mut path = Vec::<Entity>::new();
            let mut curr = end_entity;

            while curr != start_entity {
                path.push(curr);
                curr = parent_map[&curr];
            }

            path.push(start_entity);
            path.reverse();

            let start_location = building_query.get(path[0]).unwrap().1.pos().with_y(ROAD_HEIGHT + (VEHICLE_HEIGHT));
            let max_speed = VEHICLE_MAX_SPEED + rand::thread_rng().gen_range(-MAX_SPEED_VARIATION..MAX_SPEED_VARIATION);

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(VEHICLE_HEIGHT, VEHICLE_HEIGHT, VEHICLE_LENGTH)),
                    material: materials.add(Color::linear_rgb(0.5, 0.5, 0.3)),
                    transform: Transform::from_translation(start_location),
                    ..default()
                },
                Vehicle::new(path, max_speed),
                RaycastMesh::<VehicleRaycastSet>::default(),
                RaycastSource::<VehicleRaycastSet>::new_transform_empty(),
            ));
        }
    }
}

fn visualize_path(
    mut gizmos: Gizmos,
    vehicle_query: Query<&Vehicle>,
    building_query: Query<&Building>,
    inter_query: Query<&Intersection>,
    segment_query: Query<&RoadSegment>,
) {
    if let Ok(vehicle) = vehicle_query.get_single() {
        if vehicle.path.len() >= 2 {
            if let Ok(start) = building_query.get(*vehicle.path.first().unwrap()) {
                if let Ok(end) = building_query.get(*vehicle.path.last().unwrap()) {
                    gizmos.line(start.pos().with_y(5.0), end.pos().with_y(5.0), Color::WHITE);
                }
            }

            let mut prev: Option<Vec3> = None;

            for step in &vehicle.path {
                let mut pos = None;
                if let Ok(dest) = building_query.get(*step) {
                    pos = Some(dest.pos());
                } else if let Ok(edge) = segment_query.get(*step) {
                    pos = Some(edge.pos());
                } else if let Ok(node) = inter_query.get(*step) {
                    pos = Some(node.pos());
                }

                if let Some(position) = pos {
                    if let Some(previous) = prev {
                        gizmos.arrow(previous.with_y(3.0), position.with_y(3.0), Color::linear_rgb(1.0, 1.0, 0.0));
                        gizmos.circle(
                            position.with_y(3.0),
                            Dir3::from_xyz(0.0, 1.0, 0.0).unwrap(),
                            0.5,
                            Color::linear_rgb(1.0, 1.0, 0.0),
                        );
                    }
                }

                prev = pos;
            }
        }
    }
}
