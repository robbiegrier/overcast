use crate::{
    graph_events::*,
    grid::{Grid, Ground},
    road_tool::{Intersection, RoadSegment},
};
use bevy::{prelude::*, utils::HashMap};

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
                    visualize_graph.after(repair_graph),
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
    pub location: Vec3,
}

impl GraphNode {
    fn new(location: Vec3) -> Self {
        Self {
            edges: Vec::new(),
            location,
        }
    }
}

#[derive(Component, Debug)]
pub struct GraphEdge {
    pub endpoints: [Option<Entity>; 2],
    pub weight: i32,
    pub location: Vec3,
}

impl GraphEdge {
    fn new(weight: i32, location: Vec3) -> Self {
        Self {
            endpoints: [None, None],
            weight,
            location,
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
            let spawn = commands.spawn(GraphEdge::new(segment.drive_length(), segment.area.center())).id();
            graph.edges.insert(entity, spawn);
        }
    }

    // for each spawned intersection, add a node
    for &GraphNodeAddEvent(entity) in node_add_event.read() {
        if let Ok(intersection) = intersection_query.get(entity) {
            println!("the graph detected spawned node: {:?}", entity);
            let spawn = commands.spawn(GraphNode::new(intersection.area.center())).id();
            graph.nodes.insert(entity, spawn);
        }
    }
}

fn repair_graph(
    mut edge_add_event: EventReader<GraphEdgeAddEvent>,
    mut node_add_event: EventReader<GraphNodeAddEvent>,
    graph_query: Query<&Graph>,
    grid_query: Query<&Grid>,
    segment_query: Query<&RoadSegment>,
    intersection_query: Query<&Intersection>,
    mut edge_query: Query<&mut GraphEdge>,
    mut node_query: Query<&mut GraphNode>,
) {
    let graph = graph_query.single();
    let grid = grid_query.single();

    for &GraphEdgeAddEvent(entity) in edge_add_event.read() {
        let edge_entity = graph.edges[&entity];
        if let Ok(edge) = edge_query.get(edge_entity) {
            if let Ok(segment) = segment_query.get(entity) {
                if let Some(adjacent_entity) = grid.single_entity_in_area(segment.area.adjacent_top()) {
                    if let Ok(_intersection) = intersection_query.get(adjacent_entity) {
                        if let Some(node_entity) = graph.nodes.get(&adjacent_entity) {
                            if let Ok(mut node) = node_query.get_mut(*node_entity) {
                                node.edges.push(edge_entity);
                                println!("add edge {:?} to node {:?}", edge, node);
                            }
                        }
                    }
                }
                if let Some(adjacent_entity) = grid.single_entity_in_area(segment.area.adjacent_bottom()) {
                    if let Ok(_intersection) = intersection_query.get(adjacent_entity) {
                        if let Some(node_entity) = graph.nodes.get(&adjacent_entity) {
                            if let Ok(mut node) = node_query.get_mut(*node_entity) {
                                node.edges.push(edge_entity);
                                println!("add edge {:?} to node {:?}", edge, node);
                            }
                        }
                    }
                }
                if let Some(adjacent_entity) = grid.single_entity_in_area(segment.area.adjacent_left()) {
                    if let Ok(_intersection) = intersection_query.get(adjacent_entity) {
                        if let Some(node_entity) = graph.nodes.get(&adjacent_entity) {
                            if let Ok(mut node) = node_query.get_mut(*node_entity) {
                                node.edges.push(edge_entity);
                                println!("add edge {:?} to node {:?}", edge, node);
                            }
                        }
                    }
                }
                if let Some(adjacent_entity) = grid.single_entity_in_area(segment.area.adjacent_right()) {
                    if let Ok(_intersection) = intersection_query.get(adjacent_entity) {
                        if let Some(node_entity) = graph.nodes.get(&adjacent_entity) {
                            if let Ok(mut node) = node_query.get_mut(*node_entity) {
                                node.edges.push(edge_entity);
                                println!("add edge {:?} to node {:?}", edge, node);
                            }
                        }
                    }
                }
            }
        }
    }

    for &GraphNodeAddEvent(entity) in node_add_event.read() {
        let node_entity = graph.nodes[&entity];
        if let Ok(node) = node_query.get(node_entity) {
            if let Ok(intersection) = intersection_query.get(entity) {
                if let Some(adjacent_entity) = grid.single_entity_in_area(intersection.area.adjacent_bottom()) {
                    if let Ok(_segment) = segment_query.get(adjacent_entity) {
                        if let Some(edge_entity) = graph.edges.get(&adjacent_entity) {
                            if let Ok(mut edge) = edge_query.get_mut(*edge_entity) {
                                edge.endpoints[1] = Some(node_entity);
                                println!("set node {:?} as endpoint[1] of {:?}", node, edge);
                            }
                        }
                    }
                }
                if let Some(adjacent_entity) = grid.single_entity_in_area(intersection.area.adjacent_top()) {
                    if let Ok(_segment) = segment_query.get(adjacent_entity) {
                        if let Some(edge_entity) = graph.edges.get(&adjacent_entity) {
                            if let Ok(mut edge) = edge_query.get_mut(*edge_entity) {
                                edge.endpoints[0] = Some(node_entity);
                                println!("set node {:?} as endpoint[0] of {:?}", node, edge);
                            }
                        }
                    }
                }
                if let Some(adjacent_entity) = grid.single_entity_in_area(intersection.area.adjacent_left()) {
                    if let Ok(_segment) = segment_query.get(adjacent_entity) {
                        if let Some(edge_entity) = graph.edges.get(&adjacent_entity) {
                            if let Ok(mut edge) = edge_query.get_mut(*edge_entity) {
                                edge.endpoints[0] = Some(node_entity);
                                println!("set node {:?} as endpoint[0] of {:?}", node, edge);
                            }
                        }
                    }
                }
                if let Some(adjacent_entity) = grid.single_entity_in_area(intersection.area.adjacent_right()) {
                    if let Ok(_segment) = segment_query.get(adjacent_entity) {
                        if let Some(edge_entity) = graph.edges.get(&adjacent_entity) {
                            if let Ok(mut edge) = edge_query.get_mut(*edge_entity) {
                                edge.endpoints[1] = Some(node_entity);
                                println!("set node {:?} as endpoint[1] of {:?}", node, edge);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn visualize_graph(
    ground_query: Query<&GlobalTransform, With<Ground>>,
    mut gizmos: Gizmos,
    edge_query: Query<&GraphEdge>,
    node_query: Query<&GraphNode>,
) {
    let ground = ground_query.single();

    for edge in &edge_query {
        let gizmo_pos = edge.location + ground.up() * 1.0;
        gizmos.circle(gizmo_pos, ground.up(), 0.75, Color::linear_rgb(0.0, 0.0, 1.0));
        for endpoint_slot in edge.endpoints {
            if let Some(endpoint) = endpoint_slot {
                if let Ok(node) = node_query.get(endpoint) {
                    gizmos.line(node.location + ground.up() * 1.0, gizmo_pos, Color::linear_rgb(0.0, 0.0, 1.0));
                }
            }
        }
    }

    for node in &node_query {
        gizmos.circle(node.location + ground.up() * 1.0, ground.up(), 1.0, Color::WHITE);
    }
}
