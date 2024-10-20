use crate::{
    camera::PlayerCameraController,
    grid::{Grid, Ground},
    grid_area::GridArea,
    road_graph_events::*,
    schedule::UpdateStage,
    toolbar::ToolState,
};
use bevy::{prelude::*, utils::HashSet};
use rand::Rng;

pub struct BuildingToolPlugin;

impl Plugin for BuildingToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OnBuildingSpawned>().add_systems(Startup, spawn_tool).add_systems(
            Update,
            (
                (update_ground_position).in_set(UpdateStage::UpdateView),
                (adjust_tool_size, handle_tool_action, toggle_tool_mode).in_set(UpdateStage::UserInput),
            )
                .run_if(in_state(ToolState::Building)),
        );
    }
}

#[derive(Component, Debug)]
pub struct Building {
    pub area: GridArea,
    pub roads: HashSet<Entity>,
}

impl Building {
    pub fn new(area: GridArea) -> Self {
        Self {
            area,
            roads: HashSet::new(),
        }
    }

    pub fn area(&self) -> GridArea {
        self.area
    }

    pub fn pos(&self) -> Vec3 {
        self.area.center()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum BuildingToolMode {
    Spawner,
    Eraser,
}

#[derive(Component, Debug)]
pub struct BuildingTool {
    dimensions: IVec2,
    ground_position: Vec3,
    mode: BuildingToolMode,
    previous_mode: BuildingToolMode,
}

impl BuildingTool {
    fn new() -> Self {
        Self {
            dimensions: IVec2::ONE,
            ground_position: Vec3::ZERO,
            mode: BuildingToolMode::Spawner,
            previous_mode: BuildingToolMode::Spawner,
        }
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

    if let Some(cursor_position) = windows.single().cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            if let Some(distance) = ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up())) {
                let point = ray.get_point(distance);

                tool.ground_position = point;

                let area = GridArea::at(tool.ground_position, tool.dimensions.x, tool.dimensions.y);

                let mut gizmo_color = match tool.mode {
                    BuildingToolMode::Spawner => {
                        if grid_query.single().is_valid_paint_area(area) {
                            Color::linear_rgba(0.0, 1.0, 1.0, 0.8)
                        } else {
                            Color::linear_rgba(1.0, 0.0, 0.0, 0.25)
                        }
                    }
                    BuildingToolMode::Eraser => Color::linear_rgba(1.0, 1.0, 0.0, 0.8),
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
    }
}

fn toggle_tool_mode(mut query: Query<&mut BuildingTool>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut tool = query.single_mut();

    if keyboard.just_pressed(KeyCode::Digit1) {
        tool.mode = BuildingToolMode::Spawner;
    }

    if keyboard.just_pressed(KeyCode::Backspace) {
        tool.mode = match tool.mode {
            BuildingToolMode::Eraser => BuildingToolMode::Spawner,
            _ => BuildingToolMode::Eraser,
        }
    }

    if keyboard.just_pressed(KeyCode::ShiftLeft) || keyboard.just_released(KeyCode::ShiftLeft) {
        tool.mode = match tool.mode {
            BuildingToolMode::Eraser => tool.previous_mode,
            _ => {
                tool.previous_mode = tool.mode;
                BuildingToolMode::Eraser
            }
        }
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
    commands: Commands,
    query: Query<&mut BuildingTool>,
    mut grid_query: Query<&mut Grid>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    event: EventWriter<OnBuildingSpawned>,
) {
    let tool = query.single();
    let mut grid = grid_query.single_mut();

    if mouse.just_pressed(MouseButton::Left) && !keyboard.any_pressed([KeyCode::AltLeft, KeyCode::ControlLeft]) {
        match tool.mode {
            BuildingToolMode::Spawner => place_building(commands, tool, &mut grid, meshes, materials, event),
            BuildingToolMode::Eraser => erase_building(commands, tool, &mut grid),
        }
    }
}

fn place_building(
    mut commands: Commands,
    tool: &BuildingTool,
    grid: &mut Grid,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut event: EventWriter<OnBuildingSpawned>,
) {
    let area = GridArea::at(tool.ground_position, tool.dimensions.x, tool.dimensions.y);

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

fn erase_building(mut commands: Commands, tool: &BuildingTool, grid: &mut Grid) {
    let area = GridArea::at(tool.ground_position, tool.dimensions.x, tool.dimensions.y);

    for cell in area.iter() {
        if let Ok(entity_slot) = grid.entity_at(cell) {
            if let Some(entity) = entity_slot {
                grid.erase(entity);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
