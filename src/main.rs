mod assets;
mod graph;
mod graphics;
mod grid;
mod save;
mod schedule;
mod tools;
mod types;

use bevy::prelude::*;

fn main() {
    let res = std::env::current_dir();
    println!("{:?}", res.unwrap());

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(schedule::SchedulePlugin)
        .add_plugins(assets::AssetLoaderPlugin)
        .add_plugins(graph::road_graph::RoadGraphPlugin)
        .add_plugins(graphics::camera::CameraPlugin)
        .add_plugins(grid::grid::GridPlugin)
        .add_plugins(types::vehicle::VehiclePlugin)
        .add_plugins(tools::toolbar::ToolbarPlugin)
        .add_plugins(graphics::weather::WeatherPlugin)
        .add_plugins(save::save_to_disk::SavePlugin)
        .run();
}
