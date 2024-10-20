use crate::{
    building_tool::Building,
    intersection::Intersection,
    orientation::{GAxis, GDir},
    road_segment::RoadSegment,
    road_tool::ROAD_HEIGHT,
};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use if_chain::if_chain;
use rand::seq::IteratorRandom;

const VEHICLE_HEIGHT: f32 = 0.1;
const VEHICLE_LENGTH: f32 = 0.2;
const VEHICLE_MAX_SPEED: f32 = 2.0;

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_vehicle, update_vehicles, visualize_path));
    }
}

#[derive(Component, Debug)]
pub struct Vehicle {
    pub path: Vec<Entity>,
    pub path_index: usize,
    pub speed: f32,
}

impl Vehicle {
    fn new(path: Vec<Entity>) -> Self {
        Self {
            path,
            path_index: 0,
            speed: 0.0,
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

fn direction_to_intersection(segment: &RoadSegment, intersection: &Intersection) -> GDir {
    match segment.orientation {
        GAxis::Z => {
            if intersection.area.center().z > segment.area.center().z {
                GDir::North
            } else {
                GDir::South
            }
        }
        GAxis::X => {
            if intersection.area.center().x > segment.area.center().x {
                GDir::West
            } else {
                GDir::East
            }
        }
    }
}

fn get_intersection_stopping_pos(intersection: &Intersection, direction: GDir, start_pos: Vec3) -> Vec3 {
    match direction {
        GDir::North => {
            let offset =
                intersection.area.center() + Vec3::new(0.0, 0.0, (intersection.area.dimensions().y + VEHICLE_LENGTH) / 2.0);
            start_pos.with_z(offset.z)
        }
        GDir::South => {
            let offset =
                intersection.area.center() + Vec3::new(0.0, 0.0, -(intersection.area.dimensions().y + VEHICLE_LENGTH) / 2.0);
            start_pos.with_z(offset.z)
        }
        GDir::East => {
            let offset =
                intersection.area.center() + Vec3::new(-(intersection.area.dimensions().y + VEHICLE_LENGTH) / 2.0, 0.0, 0.0);
            start_pos.with_x(offset.x)
        }
        GDir::West => {
            let offset =
                intersection.area.center() + Vec3::new((intersection.area.dimensions().y + VEHICLE_LENGTH) / 2.0, 0.0, 0.0);
            start_pos.with_x(offset.x)
        }
    }
}

fn get_building_stopping_pos(building: &Building, segment: &RoadSegment, start_pos: Vec3) -> Vec3 {
    match segment.orientation {
        GAxis::Z => start_pos.with_z(building.area.center().z),
        GAxis::X => start_pos.with_x(building.area.center().x),
    }
}

fn update_vehicles(
    mut commands: Commands,
    mut vehicle_query: Query<(Entity, &mut Vehicle, &mut Transform)>,
    segment_query: Query<&RoadSegment>,
    intersection_query: Query<&Intersection>,
    building_query: Query<&Building>,
    time: Res<Time>,
    mut gizmos: Gizmos,
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

        if curr_type == StepType::Building && next_type == StepType::Road {
            if let Ok(segment) = segment_query.get(next) {
                let lane_pos = segment.get_lane_pos(transform.translation);
                transform.translation = lane_pos;
                vehicle.path_index += 1;
                println!("start on segement");
                return;
            }
        } else if curr_type == StepType::Road && next_type == StepType::Building {
            if_chain! {
                if let Ok(building) = building_query.get(next);
                if let Ok(segment) = segment_query.get(curr);
                then {
                    let stopping_pos = get_building_stopping_pos(building, segment, transform.translation);
                    gizmos.line(transform.translation, stopping_pos, Color::linear_rgb(0.0, 1.0, 1.0));
                    if transform.translation.distance(stopping_pos) < 0.01 {
                        vehicle.path_index += 1;
                        println!("reached building, done");
                        commands.entity(entity).despawn_recursive();
                        return;
                    }

                    transform.look_at(stopping_pos, Vec3::new(0.0, 1.0, 0.0));
                    let dir = (stopping_pos - transform.translation).with_y(0.0).normalize();
                    vehicle.speed = vehicle.speed.lerp(VEHICLE_MAX_SPEED, time.delta_seconds() * 0.1);
                    transform.translation += vehicle.speed * dir * time.delta_seconds();
                }
            }
        } else if curr_type == StepType::Road && next_type == StepType::Intersection {
            if_chain! {
                if let Ok(intersection) = intersection_query.get(next);
                if let Ok(segment) = segment_query.get(curr);
                then {
                    let approach_dir = direction_to_intersection(segment, intersection).inverse();
                    let stopping_pos = get_intersection_stopping_pos(intersection, approach_dir, transform.translation);
                    gizmos.line(transform.translation, stopping_pos, Color::linear_rgb(1.0, 0.0, 0.0));
                    if transform.translation.distance(stopping_pos) < 0.01 {
                        vehicle.path_index += 1;
                        println!("reached start intersect pos");
                        return;
                    }

                    transform.look_at(stopping_pos, Vec3::new(0.0, 1.0, 0.0));
                    let dir = (stopping_pos - transform.translation).with_y(0.0).normalize();
                    vehicle.speed = vehicle.speed.lerp(VEHICLE_MAX_SPEED, time.delta_seconds() * 0.1);
                    transform.translation += vehicle.speed * dir * time.delta_seconds();
                }
            }
        } else if curr_type == StepType::Intersection {
            if_chain! {
                if let Ok(intersection) = intersection_query.get(curr);
                if let Ok(next_segment) = segment_query.get(next);
                then {
                    let next_dir = direction_to_intersection(next_segment, intersection).inverse();
                    let stopping_pos = get_intersection_stopping_pos(intersection, next_dir, transform.translation);
                    let stopping_pos = next_segment.get_lane_pos(stopping_pos).with_y(transform.translation.y);
                    gizmos.line(transform.translation, stopping_pos, Color::linear_rgb(1.0, 1.0, 0.0));

                    if transform.translation.distance(stopping_pos) < 0.01 {
                         vehicle.path_index += 1;
                         println!("reached end intersect pos");
                         return;
                    }

                    transform.look_at(stopping_pos, Vec3::new(0.0, 1.0, 0.0));
                    let dir = (stopping_pos - transform.translation).with_y(0.0).normalize();
                    vehicle.speed = vehicle.speed.lerp(VEHICLE_MAX_SPEED, time.delta_seconds() * 0.1);
                    transform.translation += vehicle.speed * dir * time.delta_seconds();
                }
            }
        }
    }
}

fn spawn_vehicle(
    keyboard: Res<ButtonInput<KeyCode>>,
    building_query: Query<(Entity, &Building)>,
    segment_query: Query<(Entity, &RoadSegment)>,
    inter_query: Query<(Entity, &Intersection)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        println!("generating debug path");
        let mut rng = rand::thread_rng();
        let choose = building_query.iter().choose_multiple(&mut rng, 2);

        if choose.len() < 2 {
            println!("not enough buildings to make a path");
            return;
        }

        let (start_entity, _start) = choose[0];
        let (end_entity, _end) = choose[1];

        println!("start: {:?}; end: {:?}", start_entity, end_entity);

        let mut frontier = Vec::<Entity>::new();
        let mut visited = HashSet::<Entity>::new();
        let mut parent_map = HashMap::<Entity, Entity>::new();

        frontier.push(start_entity);

        let mut path_found = false;

        while let Some(curr) = frontier.pop() {
            visited.insert(curr);
            // if curr is destination
            if let Ok((e, dest)) = building_query.get(curr) {
                println!("curr is destination");

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
                println!("curr is edge");

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
                println!("curr is node");
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
            println!("a path was found!");
            let mut path = Vec::<Entity>::new();
            let mut curr = end_entity;

            while curr != start_entity {
                println!("path: {:?}", curr);

                path.push(curr);
                curr = parent_map[&curr];
            }

            path.push(start_entity);
            path.reverse();

            let start_location = building_query.get(path[0]).unwrap().1.pos().with_y(ROAD_HEIGHT + (VEHICLE_HEIGHT));

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(0.1, VEHICLE_HEIGHT, 0.2)),
                    material: materials.add(Color::linear_rgb(1.0, 0.8, 0.1)),
                    transform: Transform::from_translation(start_location),
                    ..default()
                },
                Vehicle::new(path),
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
