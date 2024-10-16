use crate::{
    graph_events::*,
    road_tool::{Intersection, RoadSegment},
};
use bevy::{prelude::*, utils::HashMap};
use std::sync::Arc;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GraphEdgeAddEvent>()
            .add_event::<GraphNodeAddEvent>()
            .add_event::<GraphEdgeRemoveEvent>()
            .add_event::<GraphNodeRemoveEvent>()
            .add_systems(Startup, spawn_graph)
            .add_systems(
                Update,
                (
                    remove_from_graph,
                    add_to_graph,
                    repair_graph.after(remove_from_graph).after(add_to_graph),
                ),
            );
    }
}

#[derive(Component, Debug)]
pub struct Graph {
    nodes: HashMap<Entity, Entity>,
    edges: HashMap<Entity, Entity>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }
}

#[derive(Component, Debug)]
pub struct GraphNode {
    pub edges: Vec<Entity>,
}

impl GraphNode {
    fn new() -> Self {
        Self { edges: Vec::new() }
    }
}

#[derive(Component, Debug)]
pub struct GraphEdge {
    pub endpoints: [Option<Entity>; 2],
    pub weight: i32,
}

impl GraphEdge {
    fn new(weight: i32) -> Self {
        Self {
            endpoints: [None, None],
            weight,
        }
    }
}

fn spawn_graph(mut commands: Commands) {
    commands.spawn(Graph::new());
}

fn remove_from_graph(
    mut commands: Commands,
    mut edge_remove_event: EventReader<GraphEdgeRemoveEvent>,
    mut node_remove_event: EventReader<GraphNodeRemoveEvent>,
    mut graph_query: Query<&mut Graph>,
    mut edge_query: Query<&mut GraphEdge>,
    mut node_query: Query<&mut GraphNode>,
) {
    let mut graph = graph_query.single_mut();

    // for each deleted segment, remove it and remove from attached nodes
    for &GraphEdgeRemoveEvent(entity) in edge_remove_event.read() {
        println!("the graph detected removed edge: {:?}", entity);
        let edge_entity = graph.edges[&entity];
        if let Ok(edge) = edge_query.get(edge_entity) {
            for endpoint_slot in &edge.endpoints {
                if let Some(endpoint) = endpoint_slot {
                    if let Ok(mut node) = node_query.get_mut(*endpoint) {
                        node.edges.retain(|x| *x != entity);
                    }
                }
            }
        }
        graph.edges.remove(&entity);
        commands.entity(edge_entity).despawn_recursive();
    }

    // for each deleted intersection, remove it and remove from attached edges
    for &GraphNodeRemoveEvent(entity) in node_remove_event.read() {
        println!("the graph detected removed node: {:?}", entity);
        let node_entity = graph.nodes[&entity];
        if let Ok(node) = node_query.get(node_entity) {
            for edge_entity in &node.edges {
                if let Ok(mut edge) = edge_query.get_mut(*edge_entity) {
                    for endpoint_slot in &mut edge.endpoints {
                        if let Some(endpoint) = endpoint_slot {
                            if *endpoint == entity {
                                *endpoint_slot = None;
                            }
                        }
                    }
                }
            }
        }
        graph.edges.remove(&entity);
        commands.entity(node_entity).despawn_recursive();
    }
}

fn add_to_graph(
    mut commands: Commands,
    mut edge_add_event: EventReader<GraphEdgeAddEvent>,
    mut node_add_event: EventReader<GraphNodeAddEvent>,
    mut graph_query: Query<&mut Graph>,
    segment_query: Query<&RoadSegment>,
    intersection_query: Query<&Intersection>,
) {
    let mut graph = graph_query.single_mut();

    // for each spawned segment, add an edge
    for &GraphEdgeAddEvent(entity) in edge_add_event.read() {
        if let Ok(segment) = segment_query.get(entity) {
            println!("the graph detected spawned edge: {:?}", entity);
            let spawn = commands.spawn(GraphEdge::new(segment.drive_length())).id();
            graph.edges.insert(entity, spawn);
        }
    }

    // for each spawned intersection, add a node
    for &GraphNodeAddEvent(entity) in node_add_event.read() {
        if let Ok(_intersection) = intersection_query.get(entity) {
            println!("the graph detected spawned node: {:?}", entity);
            let spawn = commands.spawn(GraphNode::new()).id();
            graph.nodes.insert(entity, spawn);
        }
    }

    // for each spawned segment edge, find and link to any nodes
    // for each spawned intersection node, find and link to any edges
}

fn repair_graph(
    mut commands: Commands,
    mut edge_remove_event: EventReader<GraphEdgeRemoveEvent>,
    mut node_remove_event: EventReader<GraphNodeRemoveEvent>,
    mut edge_add_event: EventReader<GraphEdgeAddEvent>,
    mut node_add_event: EventReader<GraphNodeAddEvent>,
) {
    for &GraphEdgeRemoveEvent(entity) in edge_remove_event.read() {
        println!("repair based on edge remove");
    }
    for &GraphNodeRemoveEvent(entity) in node_remove_event.read() {
        println!("repair based on node remove");
    }
    for &GraphEdgeAddEvent(entity) in edge_add_event.read() {
        println!("repair based on edge add");
    }
    for &GraphNodeAddEvent(entity) in node_add_event.read() {
        println!("repair based on node add");
    }
}
