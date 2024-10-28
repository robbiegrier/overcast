use crate::{graph::road_graph_events::*, grid::grid_area::*, grid::grid_cell::*, schedule::UpdateStage};
use bevy::{prelude::*, utils::HashMap};
use bevy_infinite_grid::{InfiniteGrid, InfiniteGridBundle};
use std::{f32::consts::FRAC_PI_2, fmt};

pub const GRID_RADIUS: i32 = 100;
pub const GRID_DIAMETER: i32 = GRID_RADIUS * 2;
pub const NUM_CELLS: i32 = GRID_DIAMETER * GRID_DIAMETER;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_infinite_grid::InfiniteGridPlugin)
            .add_systems(Startup, (spawn_grid, spawn_ground, spawn_grid_visualization))
            .add_systems(
                Update,
                (
                    (
                        clear_erased_objects_from_grid::<OnRoadDestroyed>,
                        clear_erased_objects_from_grid::<OnIntersectionDestroyed>,
                        clear_erased_objects_from_grid::<OnBuildingDestroyed>,
                    )
                        .in_set(UpdateStage::SoftDestroy),
                    (toggle_grid_visualization, visualize_occupancy).in_set(UpdateStage::Visualize),
                ),
            );
    }
}

#[derive(Component)]
pub struct Grid {
    entities: Vec<Option<Entity>>,
    addresses: HashMap<Entity, Vec<GridCell>>,
    center: IVec2,
}

#[derive(Debug, Clone)]
pub struct GridBoundsError;

impl fmt::Display for GridBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid index for the grid: index out of bounds")
    }
}

impl Grid {
    fn new() -> Self {
        Self {
            entities: vec![None; NUM_CELLS as usize],
            addresses: HashMap::new(),
            center: IVec2::new(GRID_RADIUS, GRID_RADIUS),
        }
    }

    fn coordinate(offset: IVec2) -> usize {
        (offset.y * GRID_DIAMETER + offset.x) as usize
    }

    pub fn entity_at(&self, cell: GridCell) -> Result<Option<Entity>, GridBoundsError> {
        let offset = self.center + cell.position;
        if offset.x >= 0 && offset.x < GRID_DIAMETER && offset.y >= 0 && offset.y < GRID_DIAMETER {
            Ok(self.entities[Grid::coordinate(offset)])
        } else {
            Err(GridBoundsError)
        }
    }

    pub fn is_occupied(&self, cell: GridCell) -> Result<bool, GridBoundsError> {
        let entity_slot = self.entity_at(cell)?;
        Ok(entity_slot.is_some())
    }

    pub fn is_valid_paint_area(&self, area: GridArea) -> bool {
        for cell in area.iter() {
            if let Ok(occupancy) = self.is_occupied(cell) {
                if occupancy {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn single_entity_in_area(&self, area: GridArea) -> Option<Entity> {
        let mut output: Option<Entity> = None;
        for cell in area.iter() {
            if let Ok(entity_slot) = self.entity_at(cell) {
                if let Some(entity) = entity_slot {
                    if let Some(unique) = output {
                        if unique != entity {
                            return None;
                        }
                    } else {
                        output = Some(entity);
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        output
    }

    pub fn mark_area_occupied(&mut self, area: GridArea, entity: Entity) {
        for cell in area.iter() {
            self.entities[Grid::coordinate(self.center + cell.position)] = Some(entity);
        }

        self.addresses.entry(entity).or_insert(Vec::new()).extend(area.iter());
    }

    pub fn erase(&mut self, entity: Entity) {
        if let Some(address_list) = self.addresses.get(&entity) {
            for cell in address_list {
                let offset = self.center + cell.position;
                self.entities[Grid::coordinate(offset)] = None;
            }

            self.addresses.remove(&entity);
        }
    }
}

fn spawn_grid(mut commands: Commands) {
    commands.spawn(Grid::new());
}

#[derive(Component)]
pub struct Ground;

fn spawn_ground(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(GRID_DIAMETER as f32, GRID_DIAMETER as f32)),
            material: materials.add(Color::srgb(0.2, 0.4, 0.2)),
            ..default()
        },
        Ground,
    ));
}

fn spawn_grid_visualization(mut commands: Commands) {
    commands.spawn(InfiniteGridBundle {
        visibility: Visibility::Hidden,
        ..default()
    });
}

fn clear_erased_objects_from_grid<E>(mut destroy_event: EventReader<E>, mut grid_query: Query<&mut Grid>)
where
    E: Event + AsRef<Entity>,
{
    let mut grid = grid_query.single_mut();

    for generic in destroy_event.read() {
        let entity: Entity = *generic.as_ref();
        println!("Clear from grid {:?}", entity);
        grid.erase(entity);
    }
}

fn toggle_grid_visualization(
    mut infinite_grid_query: Query<&mut Visibility, With<InfiniteGrid>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        let mut viz = infinite_grid_query.single_mut();
        *viz = match *viz {
            Visibility::Hidden => Visibility::Visible,
            Visibility::Visible => Visibility::Hidden,
            _ => Visibility::Hidden,
        }
    }
}

fn visualize_occupancy(
    grid_query: Query<&Grid>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    infinite_grid_query: Query<&Visibility, With<InfiniteGrid>>,
    mut gizmos: Gizmos,
) {
    let visible = infinite_grid_query.single();
    if visible == Visibility::Visible {
        let grid = grid_query.single();
        let ground = ground_query.single();
        for i in (-GRID_RADIUS)..(GRID_RADIUS) {
            for j in (-GRID_RADIUS)..(GRID_RADIUS) {
                let cell = GridCell::new(i, j);
                if let Ok(occupancy) = grid.is_occupied(cell) {
                    if occupancy {
                        gizmos.rounded_rect(
                            cell.center() + ground.up() * 0.01,
                            Quat::from_rotation_x(FRAC_PI_2),
                            Vec2::new(1.0, 1.0),
                            Color::linear_rgba(0.75, 0.0, 0.0, 1.0),
                        );
                    }
                }
            }
        }
    }
}
