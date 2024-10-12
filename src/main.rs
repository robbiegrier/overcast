mod brush;
mod camera;
mod grid;
mod weather;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(grid::GridPlugin)
        .add_plugins(brush::BrushPlugin)
        .add_plugins(weather::WeatherPlugin)
        .run();
}
