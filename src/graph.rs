use crate::road_tool::Axis;
use crate::{
    building_tool::Building,
    graph_events::*,
    grid::{Grid, Ground},
    road_segment::RoadSegment,
    road_tool::Intersection,
};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use if_chain::if_chain;

const GIZMO_HEIGHT: f32 = 0.5;
const EDGE_GIZMO_SIZE: f32 = 0.25;
const NODE_GIZMO_SIZE: f32 = 0.75;
const NODE_COLOR: Color = Color::linear_rgb(1.0, 1.0, 1.0);
const EDGE_COLOR: Color = Color::linear_rgb(0.1, 0.1, 1.0);
const DESTINATION_COLOR: Color = Color::linear_rgb(1.0, 1.0, 0.3);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum GraphRunSet {
    Mutate,
    Repair,
    Visualize,
}

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GraphEdgeAddEvent>()
            .add_event::<GraphNodeAddEvent>()
            .add_event::<GraphDestinationAddEvent>()
            .add_event::<GraphEdgeRemoveEvent>()
            .add_event::<GraphNodeRemoveEvent>()
            .add_event::<GraphNodeRepairEvent>()
            .add_event::<GraphEdgeRepairEvent>()
            .add_event::<GraphDestinationRepairEvent>()
            .init_state::<GraphVisualizationState>()
            .add_systems(Startup, spawn_graph)
            .configure_sets(
                Update,
                (GraphRunSet::Mutate, GraphRunSet::Repair, GraphRunSet::Visualize).chain(),
            )
            .add_systems(
                Update,
                (
                    (remove_from_graph, add_to_graph).in_set(GraphRunSet::Mutate),
                    repair_graph.in_set(GraphRunSet::Repair),
                    (
                        toggle_graph_visualization,
                        visualize_graph.run_if(in_state(GraphVisualizationState::Visualize)),
                    )
                        .in_set(GraphRunSet::Visualize),
                ),
            );
    }
}

#[derive(Component, Debug)]
pub struct Graph {
    pub nodes: HashMap<Entity, Entity>,
    pub edges: HashMap<Entity, Entity>,
    pub destinations: HashMap<Entity, Entity>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            destinations: HashMap::new(),
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
pub struct GraphDestination {
    pub location: Vec3,
    pub edges: Vec<Entity>,
    pub object: Entity,
}

impl GraphDestination {
    fn new(location: Vec3, object: Entity) -> Self {
        Self {
            location,
            edges: Vec::new(),
            object,
        }
    }
}

#[derive(Component, Debug)]
pub struct GraphEdge {
    pub endpoints: [Option<Entity>; 2],
    pub weight: i32,
    pub location: Vec3,
    pub orientation: Axis,
    pub destinations: Vec<Entity>,
}

impl GraphEdge {
    fn new(weight: i32, location: Vec3, orientation: Axis) -> Self {
        Self {
            endpoints: [None, None],
            weight,
            location,
            orientation,
            destinations: Vec::new(),
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
    mut destination_query: Query<&mut GraphDestination>,
) {
    let mut graph = graph_query.single_mut();

    for &GraphEdgeRemoveEvent(entity) in edge_remove_event.read() {
        if let Some(edge_entity) = graph.edges.get(&entity) {
            if let Ok(edge) = edge_query.get(*edge_entity) {
                for endpoint_slot in &edge.endpoints {
                    if let Some(endpoint) = endpoint_slot {
                        if let Ok(mut node) = node_query.get_mut(*endpoint) {
                            node.edges.retain(|x| *x != entity);
                        }
                    }
                }

                for destination_entity in &edge.destinations {
                    if let Ok(mut destination) = destination_query.get_mut(*destination_entity) {
                        destination.edges.retain(|x| *x != entity);
                    }
                }
            }
            commands.entity(*edge_entity).despawn_recursive();
        }
        graph.edges.remove(&entity);
    }

    for &GraphNodeRemoveEvent(entity) in node_remove_event.read() {
        if let Some(node_entity) = graph.nodes.get(&entity) {
            if let Ok(node) = node_query.get(*node_entity) {
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
            commands.entity(*node_entity).despawn_recursive();
        }
        graph.edges.remove(&entity);
    }
}

fn add_to_graph(
    mut commands: Commands,
    mut edge_add_event: EventReader<GraphEdgeAddEvent>,
    mut node_add_event: EventReader<GraphNodeAddEvent>,
    mut destination_add_event: EventReader<GraphDestinationAddEvent>,
    mut graph_query: Query<&mut Graph>,
    segment_query: Query<&RoadSegment>,
    intersection_query: Query<&Intersection>,
    building_query: Query<&Building>,
    mut repair_edges: EventWriter<GraphEdgeRepairEvent>,
    mut repair_nodes: EventWriter<GraphNodeRepairEvent>,
    mut repair_destinations: EventWriter<GraphDestinationRepairEvent>,
) {
    let mut graph = graph_query.single_mut();

    for &GraphEdgeAddEvent(entity) in edge_add_event.read() {
        if let Ok(segment) = segment_query.get(entity) {
            let spawn = commands
                .spawn(GraphEdge::new(
                    segment.drive_length(),
                    segment.area.center(),
                    segment.orientation,
                ))
                .id();
            graph.edges.insert(entity, spawn);
            repair_edges.send(GraphEdgeRepairEvent(entity));
        }
    }

    for &GraphNodeAddEvent(entity) in node_add_event.read() {
        if let Ok(intersection) = intersection_query.get(entity) {
            let spawn = commands.spawn(GraphNode::new(intersection.area.center())).id();
            graph.nodes.insert(entity, spawn);
            repair_nodes.send(GraphNodeRepairEvent(entity));
        }
    }

    for &GraphDestinationAddEvent(entity) in destination_add_event.read() {
        if let Ok(building) = building_query.get(entity) {
            let spawn = commands.spawn(GraphDestination::new(building.area.center(), entity)).id();
            graph.destinations.insert(entity, spawn);
            repair_destinations.send(GraphDestinationRepairEvent(entity));
        }
    }
}

fn repair_graph(
    mut edge_event: EventReader<GraphEdgeRepairEvent>,
    mut node_event: EventReader<GraphNodeRepairEvent>,
    mut destination_event: EventReader<GraphDestinationRepairEvent>,
    graph_query: Query<&Graph>,
    grid_query: Query<&Grid>,
    segment_query: Query<&RoadSegment>,
    intersection_query: Query<&Intersection>,
    building_query: Query<&Building>,
    mut edge_query: Query<&mut GraphEdge>,
    mut node_query: Query<&mut GraphNode>,
    mut destination_query: Query<&mut GraphDestination>,
) {
    let graph = graph_query.single();
    let grid = grid_query.single();

    for &GraphEdgeRepairEvent(entity) in edge_event.read() {
        if_chain! {
            if let Some(edge_entity) = graph.edges.get(&entity);
            if let Ok(mut edge) = edge_query.get_mut(*edge_entity);
            if let Ok(segment) = segment_query.get(entity);
            then {
                let mut endpoint_polarity = 1;
                for adjacent_area in segment.area.adjacent_areas() {
                    if_chain! {
                        if let Some(adjacent_entity) = grid.single_entity_in_area(adjacent_area);
                        if let Some(node_entity) = graph.nodes.get(&adjacent_entity);
                        if let Ok(mut node) = node_query.get_mut(*node_entity);
                        then {
                            node.edges.push(*edge_entity);
                            edge.endpoints[endpoint_polarity] = Some(*node_entity);
                        }
                    }
                    endpoint_polarity = (endpoint_polarity + 1) % 2;

                    for cell in adjacent_area.iter() {
                        if_chain! {
                            if let Ok(entity_slot) = grid.entity_at(cell);
                            if let Some(adjacent_entity) = entity_slot;
                            if building_query.contains(adjacent_entity);
                            if let Some(destination_entity) = graph.destinations.get(&adjacent_entity);
                            if let Ok(mut destination) = destination_query.get_mut(*destination_entity);
                            then {
                                edge.destinations.push(*destination_entity);
                                destination.edges.push(*edge_entity);
                            }
                        }
                    }
                }
            }
        }
    }

    for &GraphNodeRepairEvent(entity) in node_event.read() {
        if_chain! {
            if let Some(node_entity) = graph.nodes.get(&entity);
            if let Ok(_node) = node_query.get(*node_entity);
            if let Ok(intersection) = intersection_query.get(entity);
            then {
                let mut endpoint_polarity = 0;
                for adjacent_area in intersection.area.adjacent_areas() {
                    if_chain! {
                        if let Some(adjacent_entity) = grid.single_entity_in_area(adjacent_area);
                        if let Some(edge_entity) = graph.edges.get(&adjacent_entity);
                        if let Ok(mut edge) = edge_query.get_mut(*edge_entity);
                        then { edge.endpoints[endpoint_polarity] = Some(*node_entity); }
                    }
                    endpoint_polarity = (endpoint_polarity + 1) % 2;
                }
            }
        }
    }

    for &GraphDestinationRepairEvent(entity) in destination_event.read() {
        if_chain! {
            if let Some(destination_entity) = graph.destinations.get(&entity);
            if let Ok(building) = building_query.get(entity);
            then {
                for adjacent_area in building.area.adjacent_areas() {
                    if_chain! {
                        if let Some(adjacent_entity) = grid.single_entity_in_area(adjacent_area);
                        if let Some(edge_entity) = graph.edges.get(&adjacent_entity);
                        if let Ok(mut edge) = edge_query.get_mut(*edge_entity);
                        if let Ok(mut destination) = destination_query.get_mut(*destination_entity);
                        then {
                            edge.destinations.push(*destination_entity);
                            destination.edges.push(*edge_entity);
                        }
                    }
                }
            }
        }
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphVisualizationState {
    #[default]
    Visualize,
    Hide,
}

fn toggle_graph_visualization(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GraphVisualizationState>>,
    state: Res<State<GraphVisualizationState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        next_state.set({
            match state.get() {
                GraphVisualizationState::Hide => GraphVisualizationState::Visualize,
                GraphVisualizationState::Visualize => GraphVisualizationState::Hide,
            }
        });
    }
}

fn visualize_graph(
    ground_query: Query<&GlobalTransform, With<Ground>>,
    mut gizmos: Gizmos,
    edge_query: Query<(Entity, &GraphEdge)>,
    node_query: Query<&GraphNode>,
    destination_query: Query<&GraphDestination>,
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

        for destination_entity in &edge.destinations {
            if let Ok(destination) = destination_query.get(*destination_entity) {
                if destination.edges.contains(&entity) {
                    let center = destination.location;
                    let end = match edge.orientation {
                        Axis::X => gizmo_pos.with_x(center.x),
                        Axis::Z => gizmo_pos.with_z(center.z),
                    };
                    let building_gizmo_pos = center + ground.up() * GIZMO_HEIGHT;
                    gizmos.line_gradient(building_gizmo_pos, end, DESTINATION_COLOR, EDGE_COLOR);
                }
            }
        }
    }

    for node in &node_query {
        let pos = node.location + ground.up() * GIZMO_HEIGHT;
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
