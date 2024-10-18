use crate::graph::{GraphDestination, GraphEdge, GraphNode};
use bevy::{
    prelude::*,
    utils::{hashbrown::HashSet, HashMap},
};
use rand::seq::IteratorRandom;

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, debug_spawn_vehicle).add_systems(Update, (debug_generate_path, visualize_path));
    }
}

#[derive(Component, Debug)]
pub struct Vehicle {
    pub path: Vec<Entity>,
}

impl Vehicle {
    fn new() -> Self {
        Self { path: Vec::new() }
    }
}

fn debug_spawn_vehicle(mut commands: Commands) {
    commands.spawn(Vehicle::new());
}

fn debug_generate_path(
    keyboard: Res<ButtonInput<KeyCode>>,
    dest_query: Query<(Entity, &GraphDestination)>,
    edge_query: Query<(Entity, &GraphEdge)>,
    node_query: Query<(Entity, &GraphNode)>,
    mut vehicle_query: Query<&mut Vehicle>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        println!("generating debug path");
        let mut rng = rand::thread_rng();
        let choose = dest_query.iter().choose_multiple(&mut rng, 2);

        if choose.len() < 2 {
            println!("not enough buildings to make a path");
            return;
        }

        let (start_entity, _start) = choose[0];
        let (end_entity, _end) = choose[1];

        println!("start: {:?}; end: {:?}", start_entity, end_entity);
        if let Ok(mut vehicle) = vehicle_query.get_single_mut() {
            vehicle.path.clear();

            let mut frontier = Vec::<Entity>::new();
            let mut visited = HashSet::<Entity>::new();
            let mut parent_map = HashMap::<Entity, Entity>::new();

            frontier.push(start_entity);

            let mut path_found = false;

            while let Some(curr) = frontier.pop() {
                visited.insert(curr);
                // if curr is destination
                if let Ok((e, dest)) = dest_query.get(curr) {
                    println!("curr is destination");

                    if e == end_entity {
                        path_found = true;
                        break;
                    }
                    frontier.push(dest.edges[0]);
                    parent_map.insert(dest.edges[0], curr);
                }
                // if curr is edge
                else if let Ok((_e, edge)) = edge_query.get(curr) {
                    println!("curr is edge");

                    // if end goal is a destination here, go to it
                    if edge.destinations.contains(&end_entity) {
                        frontier.push(end_entity);
                        parent_map.insert(end_entity, curr);
                    }
                    // Add endpoints of this edge
                    else {
                        if let Some(endpoint0) = edge.endpoints[0] {
                            if let Ok((en0, _n0)) = node_query.get(endpoint0) {
                                if !visited.contains(&en0) {
                                    frontier.push(en0);
                                    parent_map.insert(en0, curr);
                                }
                            }
                        }
                        if let Some(endpoint1) = edge.endpoints[1] {
                            if let Ok((en1, _n1)) = node_query.get(endpoint1) {
                                if !visited.contains(&en1) {
                                    frontier.push(en1);
                                    parent_map.insert(en1, curr);
                                }
                            }
                        }
                    }
                }
                // if curr is a node, add connected edges
                else if let Ok((_e, node)) = node_query.get(curr) {
                    println!("curr is node");
                    for edge in &node.edges {
                        if !visited.contains(edge) {
                            frontier.push(*edge);
                            parent_map.insert(*edge, curr);
                        }
                    }
                }
            }

            if path_found {
                println!("a path was found!");
                let mut curr = end_entity;

                while curr != start_entity {
                    println!("path: {:?}", curr);

                    vehicle.path.push(curr);
                    curr = parent_map[&curr];
                }

                vehicle.path.push(start_entity);
                vehicle.path.reverse();
            }
        }
    }
}

fn visualize_path(
    mut gizmos: Gizmos,
    vehicle_query: Query<&Vehicle>,
    dest_query: Query<&GraphDestination>,
    node_query: Query<&GraphNode>,
    edge_query: Query<&GraphEdge>,
) {
    if let Ok(vehicle) = vehicle_query.get_single() {
        if vehicle.path.len() >= 2 {
            if let Ok(start) = dest_query.get(*vehicle.path.first().unwrap()) {
                if let Ok(end) = dest_query.get(*vehicle.path.last().unwrap()) {
                    gizmos.line(start.location.with_y(5.0), end.location.with_y(5.0), Color::WHITE);
                }
            }

            let mut prev: Option<Vec3> = None;

            for step in &vehicle.path {
                let mut pos = None;
                if let Ok(dest) = dest_query.get(*step) {
                    pos = Some(dest.location);
                } else if let Ok(edge) = edge_query.get(*step) {
                    pos = Some(edge.location);
                } else if let Ok(node) = node_query.get(*step) {
                    pos = Some(node.location);
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
