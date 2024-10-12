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
