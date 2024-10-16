use crate::{grid_area::GridArea, road_tool::RoadOrientation};
use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct RoadCreateEvent {
    pub area: GridArea,
    pub orientation: RoadOrientation,
}

impl RoadCreateEvent {
    pub fn new(area: GridArea, orientation: RoadOrientation) -> Self {
        Self { area, orientation }
    }
}

#[derive(Event, Debug)]
pub struct RoadSplitEvent {
    pub entity: Entity,
    pub split_area: GridArea,
}

impl RoadSplitEvent {
    pub fn new(entity: Entity, split_area: GridArea) -> Self {
        Self { entity, split_area }
    }
}

#[derive(Event, Debug)]
pub struct IntersectionCreateEvent {
    pub area: GridArea,
}

impl IntersectionCreateEvent {
    pub fn new(area: GridArea) -> Self {
        Self { area }
    }
}
