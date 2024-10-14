mod building_tool;
mod camera;
mod grid;
mod road_tool;
mod tool;
mod weather;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(grid::GridPlugin)
        .add_plugins(tool::ToolPlugin)
        .add_plugins(weather::WeatherPlugin)
        .run();
}
