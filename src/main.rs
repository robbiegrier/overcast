mod building_tool;
mod camera;
mod graph;
mod graph_events;
mod grid;
mod grid_area;
mod grid_cell;
mod road_events;
mod road_tool;
mod toolbar;
mod weather;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(grid::GridPlugin)
        .add_plugins(graph::GraphPlugin)
        .add_plugins(toolbar::ToolbarPlugin)
        .add_plugins(weather::WeatherPlugin)
        .run();
}
