use crate::{
    grid::{grid_area::*, orientation::GAxis},
    save::save_events::*,
    schedule::UpdateStage,
    tools::{
        building_tool::RequestBuilding,
        road_events::{RequestIntersection, RequestRoad},
    },
    types::{building::*, intersection::Intersection, road_segment::RoadSegment},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

const SAVEFILE: &str = "saves/world.json";

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveRequest>()
            .add_systems(PostStartup, load_from_disk)
            .add_systems(Update, (save_on_key_press.in_set(UpdateStage::UserInput), save_to_disk));
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SaveObject {
    buildings: Vec<GridArea>,
    intersections: Vec<GridArea>,
    roads: Vec<(GridArea, GAxis)>,
}

impl SaveObject {
    pub fn new() -> Self {
        Self {
            buildings: Vec::new(),
            intersections: Vec::new(),
            roads: Vec::new(),
        }
    }
}

pub fn load_from_disk(
    mut building_event: EventWriter<RequestBuilding>,
    mut inter_event: EventWriter<RequestIntersection>,
    mut segment_event: EventWriter<RequestRoad>,
) {
    if let Ok(file) = File::open(SAVEFILE) {
        let reader = BufReader::new(file);
        if let Ok(save_data) = serde_json::from_reader::<std::io::BufReader<File>, SaveObject>(reader) {
            for area in save_data.buildings {
                building_event.send(RequestBuilding::new(area));
            }

            for area in save_data.intersections {
                inter_event.send(RequestIntersection::new(area));
            }

            for (area, orient) in save_data.roads {
                segment_event.send(RequestRoad::new(area, orient));
            }

            println!("Loaded the game from {:?}", SAVEFILE);
        }
    }
}

pub fn save_on_key_press(keyboard: Res<ButtonInput<KeyCode>>, mut event: EventWriter<SaveRequest>) {
    if keyboard.just_pressed(KeyCode::F5) {
        event.send(SaveRequest);
    }
}

pub fn save_to_disk(
    building_query: Query<&Building>,
    segment_query: Query<&RoadSegment>,
    inter_query: Query<&Intersection>,
    mut event: EventReader<SaveRequest>,
) {
    for _ in event.read() {
        let mut save_data = SaveObject::new();

        for building in &building_query {
            save_data.buildings.push(building.area());
        }

        for inter in &inter_query {
            save_data.intersections.push(inter.area());
        }

        for segment in &segment_query {
            save_data.roads.push((segment.area(), segment.orientation));
        }

        if std::fs::create_dir_all("saves").is_ok() {
            if let Ok(file) = File::create(SAVEFILE) {
                let mut writer = BufWriter::new(file);
                if serde_json::to_writer(&mut writer, &save_data).is_ok() && writer.flush().is_ok() {
                    println!("Saved the game to {:?}", SAVEFILE);
                }
            }
        }
    }
}
