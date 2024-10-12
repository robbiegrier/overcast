use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGrid, InfiniteGridBundle};

pub const GRID_RADIUS: f32 = 100.0;
pub const GRID_DIAMETER: f32 = GRID_RADIUS * 2.0;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_infinite_grid::InfiniteGridPlugin)
            .add_systems(Startup, (spawn_ground, spawn_grid_visualization))
            .add_systems(Update, toggle_grid_visualization);
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

    pub fn max_corner(&self) -> Vec3 {
        Vec3::new(self.position.x as f32 + 1.0, 0.0, self.position.y as f32 + 1.0)
    }

    pub fn min_corner(&self) -> Vec3 {
        Vec3::new(self.position.x as f32, 0.0, self.position.y as f32)
    }
}

#[derive(Component)]
pub struct Ground;

fn spawn_ground(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(GRID_DIAMETER, GRID_DIAMETER)),
            material: materials.add(Color::srgb(0.2, 0.4, 0.2)),
            ..default()
        },
        Ground,
    ));
}

fn spawn_grid_visualization(mut commands: Commands) {
    commands.spawn(InfiniteGridBundle::default());
}

fn toggle_grid_visualization(mut query: Query<&mut Visibility, With<InfiniteGrid>>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        let mut viz = query.single_mut();
        *viz = match *viz {
            Visibility::Hidden => Visibility::Visible,
            Visibility::Visible => Visibility::Hidden,
            _ => Visibility::Hidden,
        }
    }
}
