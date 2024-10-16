use crate::{
    graph_events::*,
    grid::{Grid, Ground},
    road_segment::RoadSegment,
    road_tool::Intersection,
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
        // println!("the graph detected removed edge: {:?}", entity);
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
        // println!("the graph detected removed node: {:?}", entity);
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
            // println!("the graph detected spawned edge: {:?}", entity);
            let spawn = commands.spawn(GraphEdge::new(segment.drive_length(), segment.area.center())).id();
            graph.edges.insert(entity, spawn);
        }
    }

    // for each spawned intersection, add a node
    for &GraphNodeAddEvent(entity) in node_add_event.read() {
        if let Ok(intersection) = intersection_query.get(entity) {
            // println!("the graph detected spawned node: {:?}", entity);
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
        if let Ok(mut edge) = edge_query.get_mut(edge_entity) {
            if let Ok(segment) = segment_query.get(entity) {
                let mut endpoint_polarity = 1;
                for adjacent_area in segment.area.adjacent_areas() {
                    if let Some(adjacent_entity) = grid.single_entity_in_area(adjacent_area) {
                        if let Ok(_intersection) = intersection_query.get(adjacent_entity) {
                            if let Some(node_entity) = graph.nodes.get(&adjacent_entity) {
                                if let Ok(mut node) = node_query.get_mut(*node_entity) {
                                    node.edges.push(edge_entity);
                                    edge.endpoints[endpoint_polarity] = Some(*node_entity);
                                    // println!("add edge to node");
                                }
                            }
                        }
                    }
                    endpoint_polarity = (endpoint_polarity + 1) % 2;
                }
            }
        }
    }

    for &GraphNodeAddEvent(entity) in node_add_event.read() {
        let node_entity = graph.nodes[&entity];
        if let Ok(_node) = node_query.get(node_entity) {
            if let Ok(intersection) = intersection_query.get(entity) {
                let mut endpoint_polarity = 0;
                for adjacent_area in intersection.area.adjacent_areas() {
                    if let Some(adjacent_entity) = grid.single_entity_in_area(adjacent_area) {
                        if let Ok(_segment) = segment_query.get(adjacent_entity) {
                            if let Some(edge_entity) = graph.edges.get(&adjacent_entity) {
                                if let Ok(mut edge) = edge_query.get_mut(*edge_entity) {
                                    edge.endpoints[endpoint_polarity] = Some(node_entity);
                                    // println!("set node {:?} as endpoint of {:?}", node, edge);
                                }
                            }
                        }
                    }
                    endpoint_polarity = (endpoint_polarity + 1) % 2;
                }
            }
        }
    }
}

const GIZMO_HEIGHT: f32 = 0.5;
const EDGE_GIZMO_SIZE: f32 = 0.25;
const NODE_GIZMO_SIZE: f32 = 0.75;
const NODE_COLOR: Color = Color::linear_rgb(1.0, 1.0, 1.0);
const EDGE_COLOR: Color = Color::linear_rgb(0.0, 0.0, 1.0);

fn visualize_graph(
    ground_query: Query<&GlobalTransform, With<Ground>>,
    mut gizmos: Gizmos,
    edge_query: Query<(Entity, &GraphEdge)>,
    node_query: Query<&GraphNode>,
) {
    let ground = ground_query.single();

    for (entity, edge) in &edge_query {
        let gizmo_pos = edge.location + ground.up() * GIZMO_HEIGHT;
        let scale = EDGE_GIZMO_SIZE * edge.weight as f32 / 20.0;
        gizmos.circle(gizmo_pos, ground.up(), scale, EDGE_COLOR);
        for endpoint_slot in edge.endpoints {
            if let Some(endpoint) = endpoint_slot {
                if let Ok(node) = node_query.get(endpoint) {
                    if node.edges.contains(&entity) {
                        let dir = (edge.location - node.location).normalize();
                        let start = (node.location + ground.up() * GIZMO_HEIGHT) + (dir * NODE_GIZMO_SIZE);
                        let end = gizmo_pos + (-dir * scale as f32);
                        gizmos.line_gradient(start, end, NODE_COLOR, EDGE_COLOR);
                    }
                }
            }
        }
    }

    for node in &node_query {
        let pos = node.location + ground.up() * GIZMO_HEIGHT;
        // gizmos.rect(pos, ground.up(), NODE_GIZMO_SIZE, NODE_COLOR);
        gizmos.rounded_rect(
            pos,
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            Vec2::new(NODE_GIZMO_SIZE * 2.0, NODE_GIZMO_SIZE * 2.0),
            NODE_COLOR,
        );
        // for edge_entity in &node.edges {
        //     if let Ok((_, edge)) = edge_query.get(*edge_entity) {
        //         let dir = (edge.location - node.location).normalize();
        //         let start = (edge.location + ground.up() * GIZMO_HEIGHT) + (-dir * EDGE_GIZMO_SIZE);
        //         let end = pos + (dir * NODE_GIZMO_SIZE);
        //         gizmos.line(
        //             start + Vec3::new(0.0, 1.0, 0.0),
        //             end + Vec3::new(0.0, 1.0, 0.0),
        //             Color::linear_rgb(1.0, 1.0, 0.0),
        //         );
        //     }
        // }
    }
}
