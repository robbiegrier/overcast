mod graph;
mod graphics;
mod grid;
mod save;
mod schedule;
mod tools;
mod types;
mod ui;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(schedule::SchedulePlugin)
        .add_plugins(graph::road_graph::RoadGraphPlugin)
        .add_plugins(graphics::camera::CameraPlugin)
        .add_plugins(graphics::models::ModelPlugin)
        .add_plugins(grid::grid::GridPlugin)
        .add_plugins(types::vehicle::VehiclePlugin)
        .add_plugins(tools::toolbar::ToolbarPlugin)
        .add_plugins(graphics::weather::WeatherPlugin)
        .add_plugins(save::save::SavePlugin)
        .add_plugins(ui::egui::UiPlugin)
        .run();
}
