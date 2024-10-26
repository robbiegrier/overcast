use crate::{
    graph::road_graph_events::*, graphics::camera::*, grid::grid::*, grid::grid_area::*, schedule::UpdateStage,
    tools::toolbar::ToolState, types::building::*, types::intersection::*, types::road_segment::*,
};
use bevy::prelude::*;

pub struct EraserToolPlugin;

impl Plugin for EraserToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tool).add_systems(
            Update,
            (
                (
                    (update_ground_position).in_set(UpdateStage::UpdateView),
                    (adjust_tool_size, handle_tool_action).in_set(UpdateStage::UserInput),
                )
                    .run_if(in_state(ToolState::Eraser)),
                (
                    despawn_erased_entities::<OnRoadDestroyed>,
                    despawn_erased_entities::<OnIntersectionDestroyed>,
                    despawn_erased_entities::<OnBuildingDestroyed>,
                )
                    .in_set(UpdateStage::DestroyEntities),
            ),
        );
    }
}

#[derive(Component, Debug)]
pub struct EraserTool {
    dimensions: IVec2,
    ground_position: Vec3,
}

impl EraserTool {
    fn new() -> Self {
        Self {
            dimensions: IVec2::ONE,
            ground_position: Vec3::ZERO,
        }
    }
}

fn spawn_tool(mut commands: Commands) {
    commands.spawn(EraserTool::new());
}

fn update_ground_position(
    camera_query: Query<(&Camera, &PlayerCameraController, &GlobalTransform)>,
    mut tool_query: Query<&mut EraserTool>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
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
        let area = GridArea::at(tool.ground_position, tool.dimensions.x, tool.dimensions.y);
        let mut gizmo_color = Color::linear_rgba(1.0, 1.0, 0.0, 0.8);

        if controller.is_moving() {
            gizmo_color = gizmo_color.with_alpha(0.25);
        }

        gizmos.cuboid(
            Transform::from_translation(area.center().with_y(0.5)).with_scale(Vec3::new(
                area.dimensions().x,
                1.0,
                area.dimensions().y,
            )),
            gizmo_color,
        );
    }
}

fn adjust_tool_size(mut query: Query<&mut EraserTool>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut tool = query.single_mut();

    if keyboard.just_pressed(KeyCode::KeyR) {
        tool.dimensions.x += 1;
        tool.dimensions.y += 1;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        tool.dimensions.x -= 1;
        tool.dimensions.y -= 1;
    }

    tool.dimensions = tool.dimensions.max(IVec2::new(1, 1));
}

fn handle_tool_action(
    query: Query<&mut EraserTool>,
    grid_query: Query<&Grid>,
    segment_query: Query<&RoadSegment>,
    inter_query: Query<&Intersection>,
    building_query: Query<&Building>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut segment_event: EventWriter<OnRoadDestroyed>,
    mut inter_event: EventWriter<OnIntersectionDestroyed>,
    mut building_event: EventWriter<OnBuildingDestroyed>,
) {
    let tool = query.single();
    let grid = grid_query.single();

    if mouse.just_pressed(MouseButton::Left) && !keyboard.any_pressed([KeyCode::AltLeft, KeyCode::ControlLeft]) {
        let area = GridArea::at(tool.ground_position, tool.dimensions.x, tool.dimensions.y);

        for cell in area.iter() {
            if let Ok(Some(entity)) = grid.entity_at(cell) {
                if building_query.contains(entity) {
                    building_event.send(OnBuildingDestroyed(entity));
                } else if segment_query.contains(entity) {
                    segment_event.send(OnRoadDestroyed(entity));
                } else if inter_query.contains(entity) {
                    inter_event.send(OnIntersectionDestroyed(entity));
                }
            }
        }
    }
}

fn despawn_erased_entities<E>(mut event_reader: EventReader<E>, mut commands: Commands)
where
    E: Event + AsRef<Entity>,
{
    for generic in event_reader.read() {
        let entity: Entity = *generic.as_ref();
        println!("Finally remove entity {:?}", entity);
        commands.entity(entity).despawn_recursive();
    }
}
