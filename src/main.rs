mod building_tool;
mod camera;
mod eraser_tool;
mod grid;
mod grid_area;
mod grid_cell;
mod intersection;
mod orientation;
mod road_events;
mod road_graph;
mod road_graph_events;
mod road_segment;
mod road_tool;
mod schedule;
mod toolbar;
mod vehicle;
mod weather;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(schedule::SchedulePlugin)
        .add_plugins(road_graph::RoadGraphPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(grid::GridPlugin)
        .add_plugins(vehicle::VehiclePlugin)
        .add_plugins(toolbar::ToolbarPlugin)
        .add_plugins(weather::WeatherPlugin)
        .run();
}
