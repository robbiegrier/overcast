use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGrid, InfiniteGridBundle};

pub const GRID_RADIUS: i32 = 100;
pub const GRID_DIAMETER: i32 = GRID_RADIUS * 2;

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
    occupancy: [[bool; GRID_DIAMETER as usize]; GRID_DIAMETER as usize],
    center: IVec2,
}

impl Grid {
    fn new() -> Self {
        Self {
            occupancy: [[false; GRID_DIAMETER as usize]; GRID_DIAMETER as usize],
            center: IVec2::new(GRID_RADIUS, GRID_RADIUS),
        }
    }

    pub fn is_occupied(&self, cell: GridCell) -> Option<bool> {
        let offset = self.center + cell.position;
        if offset.x >= 0 && offset.x < GRID_DIAMETER && offset.y >= 0 && offset.y < GRID_DIAMETER {
            Some(self.occupancy[offset.x as usize][offset.y as usize])
        } else {
            None
        }
    }

    pub fn is_valid_paint_area(&self, area: GridArea) -> bool {
        for cell in area.iter() {
            if let Some(occupancy) = self.is_occupied(cell) {
                if occupancy {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn mark_occupied(&mut self, cell: GridCell) {
        let offset = self.center + cell.position;
        if offset.x >= 0 && offset.x < GRID_DIAMETER && offset.y >= 0 && offset.y < GRID_DIAMETER {
            self.occupancy[offset.x as usize][offset.y as usize] = true;
        }
    }

    pub fn mark_area_occupied(&mut self, area: GridArea) {
        for cell in area.iter() {
            self.mark_occupied(cell);
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
                if let Some(occupancy) = grid.is_occupied(cell) {
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
