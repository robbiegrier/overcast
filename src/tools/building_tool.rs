use crate::{
    graph::road_graph_events::*, graphics::camera::*, grid::grid::*, grid::grid_area::*, schedule::UpdateStage,
    tools::toolbar::ToolState, types::building::*,
};
use bevy::prelude::*;
use rand::Rng;

pub struct BuildingToolPlugin;

impl Plugin for BuildingToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RequestBuilding>().add_systems(Startup, spawn_tool).add_systems(
            Update,
            (
                (
                    (update_ground_position).in_set(UpdateStage::UpdateView),
                    (adjust_tool_size, handle_tool_action).in_set(UpdateStage::UserInput),
                )
                    .run_if(in_state(ToolState::Building)),
                (spawn_buildings).in_set(UpdateStage::Spawning),
            ),
        );
    }
}

#[derive(Component, Debug)]
pub struct BuildingTool {
    dimensions: IVec2,
    ground_position: Vec3,
}

impl BuildingTool {
    fn new() -> Self {
        Self {
            dimensions: IVec2::ONE,
            ground_position: Vec3::ZERO,
        }
    }
}

#[derive(Event, Debug)]
pub struct RequestBuilding {
    pub area: GridArea,
}

impl RequestBuilding {
    pub fn new(area: GridArea) -> Self {
        Self { area }
    }
}

fn spawn_tool(mut commands: Commands) {
    commands.spawn(BuildingTool::new());
}

fn update_ground_position(
    camera_query: Query<(&Camera, &PlayerCameraController, &GlobalTransform)>,
    mut tool_query: Query<&mut BuildingTool>,
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

        let area = GridArea::at(tool.ground_position, tool.dimensions.x, tool.dimensions.y);

        let mut gizmo_color = if grid_query.single().is_valid_paint_area(area) {
            Color::linear_rgba(0.0, 1.0, 1.0, 0.8)
        } else {
            Color::linear_rgba(1.0, 0.0, 0.0, 0.25)
        };

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

fn adjust_tool_size(mut query: Query<&mut BuildingTool>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut tool = query.single_mut();

    if keyboard.just_pressed(KeyCode::KeyR) {
        tool.dimensions.x += 1;
        tool.dimensions.y += 1;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        tool.dimensions.x -= 1;
        tool.dimensions.y -= 1;
    }

    if keyboard.just_pressed(KeyCode::BracketRight) {
        tool.dimensions.x += 1;
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        tool.dimensions.x -= 1;
    }

    if keyboard.just_pressed(KeyCode::Equal) {
        tool.dimensions.y += 1;
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        tool.dimensions.y -= 1;
    }

    tool.dimensions = tool.dimensions.max(IVec2::new(1, 1));
}

fn handle_tool_action(
    query: Query<&mut BuildingTool>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut builder: EventWriter<RequestBuilding>,
) {
    let tool = query.single();

    if mouse.just_pressed(MouseButton::Left) && !keyboard.any_pressed([KeyCode::AltLeft, KeyCode::ControlLeft]) {
        let area = GridArea::at(tool.ground_position, tool.dimensions.x, tool.dimensions.y);
        builder.send(RequestBuilding::new(area));
    }
}

fn spawn_buildings(
    mut commands: Commands,
    mut grid_query: Query<&mut Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut event: EventWriter<OnBuildingSpawned>,
    mut builder: EventReader<RequestBuilding>,
) {
    let mut grid = grid_query.single_mut();

    for &RequestBuilding { area } in builder.read() {
        let rheight = rand::thread_rng().gen_range(0.5..6.0);
        let rgray = rand::thread_rng().gen_range(0.05..0.25);
        let crop = 0.5;

        if grid.is_valid_paint_area(area) {
            let model = PbrBundle {
                mesh: meshes.add(Cuboid::new(area.dimensions().x - crop, rheight, area.dimensions().y - crop)),
                material: materials.add(Color::linear_rgb(rgray, rgray, rgray)),
                transform: Transform::from_translation(area.center().with_y(rheight / 2.0)),
                ..default()
            };

            let entity = commands.spawn((model, Building::new(area))).id();
            grid.mark_area_occupied(area, entity);
            event.send(OnBuildingSpawned(entity));
        }
    }
}
