use crate::{
    graph::road_graph_events::*, grid::grid::Grid, schedule::UpdateStage, types::building::*,
    types::intersection::Intersection, types::road_segment::RoadSegment,
};
use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphVisualizationState {
    Visualize,
    #[default]
    Hide,
}

pub struct RoadGraphPlugin;

impl Plugin for RoadGraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GraphVisualizationState>()
            .add_event::<OnRoadSpawned>()
            .add_event::<OnIntersectionSpawned>()
            .add_event::<OnBuildingSpawned>()
            .add_event::<OnRoadDestroyed>()
            .add_event::<OnIntersectionDestroyed>()
            .add_event::<OnBuildingDestroyed>()
            .add_systems(
                Update,
                (
                    (toggle_graph_visualization).in_set(UpdateStage::UserInput),
                    (
                        add_roads_to_graph,
                        add_intersections_to_graph,
                        add_buildings_to_graph,
                        remove_roads_from_graph,
                        remove_intersections_from_graph,
                        remove_buildings_from_graph,
                    )
                        .in_set(UpdateStage::Analyze),
                    (visualize_segments, visualize_intersections, visualize_buildings)
                        .in_set(UpdateStage::Visualize)
                        .run_if(in_state(GraphVisualizationState::Visualize)),
                ),
            );
    }
}

pub fn add_roads_to_graph(
    mut event: EventReader<OnRoadSpawned>,
    grid_query: Query<&Grid>,
    mut segment_query: Query<&mut RoadSegment>,
    mut inter_query: Query<&mut Intersection>,
    mut building_query: Query<&mut Building>,
) {
    let grid = grid_query.single();

    for &OnRoadSpawned(entity) in event.read() {
        if let Ok(mut segment) = segment_query.get_mut(entity) {
            for (adj_area, gdir) in segment.area().adjacent_areas() {
                if let Some(adj) = grid.single_entity_in_area(adj_area) {
                    if let Ok(mut inter) = inter_query.get_mut(adj) {
                        segment.ends[gdir.binary_index()] = Some(adj);
                        inter.roads[gdir.inverse().index()] = Some(entity);
                    }
                }

                for cell in adj_area.iter() {
                    if let Ok(Some(adj)) = grid.entity_at(cell) {
                        if let Ok(mut building) = building_query.get_mut(adj) {
                            segment.dests.insert(adj);
                            building.roads.insert(entity);
                        }
                    }
                }
            }
        }
    }
}

pub fn add_intersections_to_graph(
    mut event: EventReader<OnIntersectionSpawned>,
    grid_query: Query<&Grid>,
    mut segment_query: Query<&mut RoadSegment>,
    mut inter_query: Query<&mut Intersection>,
) {
    let grid = grid_query.single();

    for &OnIntersectionSpawned(entity) in event.read() {
        if let Ok(mut inter) = inter_query.get_mut(entity) {
            for (adj_area, gdir) in inter.area().adjacent_areas() {
                if let Some(adj) = grid.single_entity_in_area(adj_area) {
                    if let Ok(mut segment) = segment_query.get_mut(adj) {
                        inter.roads[gdir.index()] = Some(adj);
                        segment.ends[gdir.inverse().binary_index()] = Some(entity);
                    }
                }
            }
        }
    }
}

pub fn add_buildings_to_graph(
    mut event: EventReader<OnBuildingSpawned>,
    grid_query: Query<&Grid>,
    mut segment_query: Query<&mut RoadSegment>,
    mut building_query: Query<&mut Building>,
) {
    let grid = grid_query.single();

    for &OnBuildingSpawned(entity) in event.read() {
        if let Ok(mut building) = building_query.get_mut(entity) {
            for (adj_area, _gdir) in building.area().adjacent_areas() {
                if let Some(adj) = grid.single_entity_in_area(adj_area) {
                    if let Ok(mut segment) = segment_query.get_mut(adj) {
                        building.roads.insert(adj);
                        segment.dests.insert(entity);
                    }
                }
            }
        }
    }
}

pub fn remove_roads_from_graph(
    mut event: EventReader<OnRoadDestroyed>,
    segment_query: Query<&RoadSegment>,
    mut inter_query: Query<&mut Intersection>,
    mut building_query: Query<&mut Building>,
) {
    for &OnRoadDestroyed(entity) in event.read() {
        if let Ok(segment) = segment_query.get(entity) {
            for slot in &segment.ends {
                if let Some(end) = slot {
                    if let Ok(mut inter) = inter_query.get_mut(*end) {
                        for road in &mut inter.roads {
                            if let Some(other_segment) = road {
                                if *other_segment == entity {
                                    *road = None;
                                }
                            }
                        }
                    }
                }
            }

            for dest in &segment.dests {
                if let Ok(mut building) = building_query.get_mut(*dest) {
                    building.roads.remove(&entity);
                }
            }
        }
    }
}

pub fn remove_intersections_from_graph(
    mut event: EventReader<OnIntersectionDestroyed>,
    inter_query: Query<&mut Intersection>,
    mut segment_query: Query<&mut RoadSegment>,
) {
    for &OnIntersectionDestroyed(entity) in event.read() {
        if let Ok(inter) = inter_query.get(entity) {
            for slot in &inter.roads {
                if let Some(road) = slot {
                    if let Ok(mut segment) = segment_query.get_mut(*road) {
                        for end in &mut segment.ends {
                            if *end == Some(entity) {
                                *end = None;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn remove_buildings_from_graph(
    mut event: EventReader<OnBuildingDestroyed>,
    building_query: Query<&Building>,
    mut segment_query: Query<&mut RoadSegment>,
) {
    for &OnBuildingDestroyed(entity) in event.read() {
        if let Ok(building) = building_query.get(entity) {
            for road in &building.roads {
                if let Ok(mut segment) = segment_query.get_mut(*road) {
                    segment.dests.remove(&entity);
                }
            }
        }
    }
}

const VIZ_Y: f32 = 1.0;
const CONNECT_COLOR: Color = Color::linear_rgb(1.0, 1.0, 1.0);
const SEGMENT_COLOR: Color = Color::linear_rgb(0.0, 0.0, 1.0);
const INTER_COLOR: Color = Color::linear_rgb(1.0, 1.0, 0.0);
const BUILDING_COLOR: Color = Color::linear_rgb(0.0, 1.0, 1.0);
const CONNECT_RADIUS: f32 = 0.1;
const SEGMENT_RADIUS: f32 = 0.2;
const INTER_RADIUS: f32 = 0.4;
const BUILDING_RADIUS: f32 = 0.3;

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

pub fn visualize_segments(
    segment_query: Query<&RoadSegment>,
    inter_query: Query<&Intersection>,
    building_query: Query<&Building>,
    mut gizmos: Gizmos,
) {
    for segment in &segment_query {
        let start = segment.pos().with_y(VIZ_Y);
        gizmos.circle(start, Dir3::Y, SEGMENT_RADIUS, SEGMENT_COLOR);

        for end in segment.ends {
            if let Some(inter_ent) = end {
                if let Ok(inter) = inter_query.get(inter_ent) {
                    let end = inter.pos().with_y(VIZ_Y);
                    let vec = end - start;
                    let dir = vec.normalize();
                    let connect = start + (vec / 2.0);
                    gizmos.rounded_rect(
                        connect,
                        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                        Vec2::new(CONNECT_RADIUS * 2.0, CONNECT_RADIUS * 2.0),
                        CONNECT_COLOR,
                    );
                    gizmos.line_gradient(
                        start + dir * SEGMENT_RADIUS,
                        connect - dir * CONNECT_RADIUS,
                        SEGMENT_COLOR,
                        CONNECT_COLOR,
                    );
                }
            }
        }

        for dest in &segment.dests {
            if let Ok(building) = building_query.get(*dest) {
                let end = building.pos().with_y(VIZ_Y);
                let vec = end - start;
                let dir = vec.normalize();
                let connect = start + (vec / 2.0);
                gizmos.rounded_rect(
                    connect,
                    Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                    Vec2::new(CONNECT_RADIUS * 2.0, CONNECT_RADIUS * 2.0),
                    CONNECT_COLOR,
                );
                gizmos.line_gradient(
                    start + dir * SEGMENT_RADIUS,
                    connect - dir * CONNECT_RADIUS,
                    SEGMENT_COLOR,
                    CONNECT_COLOR,
                );
            }
        }
    }
}

pub fn visualize_intersections(segment_query: Query<&RoadSegment>, inter_query: Query<&Intersection>, mut gizmos: Gizmos) {
    for inter in &inter_query {
        let start = inter.pos().with_y(VIZ_Y);
        gizmos.circle(start, Dir3::Y, INTER_RADIUS, INTER_COLOR);

        for slot in &inter.roads {
            if let Some(road) = slot {
                if let Ok(segment) = segment_query.get(*road) {
                    let end = segment.pos().with_y(VIZ_Y);
                    let vec = end - start;
                    let dir = (end - start).normalize();
                    let connect = start + (vec / 2.0);
                    gizmos.line_gradient(
                        start + dir * INTER_RADIUS,
                        connect - dir * CONNECT_RADIUS,
                        INTER_COLOR,
                        CONNECT_COLOR,
                    );
                }
            }
        }
    }
}

pub fn visualize_buildings(building_query: Query<&Building>, segment_query: Query<&RoadSegment>, mut gizmos: Gizmos) {
    for building in &building_query {
        let start = building.pos().with_y(VIZ_Y);
        gizmos.rounded_rect(
            start,
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            Vec2::new(building.area.dimensions().x, building.area.dimensions().y),
            BUILDING_COLOR,
        );

        for road in &building.roads {
            if let Ok(segment) = segment_query.get(*road) {
                let end = segment.pos().with_y(VIZ_Y);
                let vec = end - start;
                let dir = (end - start).normalize();
                let connect = start + (vec / 2.0);
                gizmos.line_gradient(
                    start + dir * BUILDING_RADIUS,
                    connect - dir * CONNECT_RADIUS,
                    BUILDING_COLOR,
                    CONNECT_COLOR,
                );
            }
        }
    }
}
