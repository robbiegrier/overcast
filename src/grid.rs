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
            .add_systems(Update, (toggle_grid_visualization, visualize_occupancy));
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

#[derive(Copy, Clone, Debug)]
pub struct GridArea {
    min: GridCell,
    max: GridCell,
}

impl GridArea {
    pub fn at(location: Vec3, width: i32, height: i32) -> Self {
        let hover_cell = GridCell::at(location);
        let mut min = hover_cell.position.clone();
        let mut max = hover_cell.position.clone();

        if width % 2 != 0 {
            let radius = (width - 1) / 2;
            min.x -= radius;
            max.x += radius;
        } else {
            let radius = width / 2;
            min.x -= radius - 1;
            max.x += radius;
        }

        if height % 2 != 0 {
            let radius = (height - 1) / 2;
            min.y -= radius;
            max.y += radius;
        } else {
            let radius = height / 2;
            min.y -= radius - 1;
            max.y += radius;
        }

        Self {
            min: GridCell::new(min.x, min.y),
            max: GridCell::new(max.x, max.y),
        }
    }

    pub fn center(&self) -> Vec3 {
        let center_2d = (self.min.min_corner() + self.max.max_corner()) / 2.0;
        Vec3::new(center_2d.x, 0.0, center_2d.z)
    }

    pub fn dimensions(&self) -> Vec2 {
        let max = self.max.max_corner();
        let min = self.min.min_corner();
        Vec2 {
            x: max.x - min.x,
            y: max.z - min.z,
        }
    }

    pub fn iter(&self) -> GridAreaIterator {
        GridAreaIterator {
            area: self,
            current: GridCell::new(self.min.position.x - 1, self.min.position.y),
        }
    }
}

pub struct GridAreaIterator<'a> {
    area: &'a GridArea,
    current: GridCell,
}

impl<'a> Iterator for GridAreaIterator<'a> {
    type Item = GridCell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.position.x < self.area.max.position.x {
            self.current = GridCell::new(self.current.position.x + 1, self.current.position.y);
            Some(self.current)
        } else if self.current.position.y < self.area.max.position.y {
            self.current = GridCell::new(self.area.min.position.x, self.current.position.y + 1);
            Some(self.current)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GridCell {
    position: IVec2,
}

impl GridCell {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            position: IVec2::new(x, y),
        }
    }

    pub fn at(location: Vec3) -> Self {
        let position = IVec2::new(location.x.floor() as i32, location.z.floor() as i32);
        Self { position }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(self.position.x as f32 + 0.5, 0.0, self.position.y as f32 + 0.5)
    }

    pub fn max_corner(&self) -> Vec3 {
        Vec3::new(self.position.x as f32 + 1.0, 0.0, self.position.y as f32 + 1.0)
    }

    pub fn min_corner(&self) -> Vec3 {
        Vec3::new(self.position.x as f32, 0.0, self.position.y as f32)
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
