use crate::{grid_area::GridArea, road_tool::Axis};
use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct RoadCreateEvent {
    pub area: GridArea,
    pub orientation: Axis,
}

impl RoadCreateEvent {
    pub fn new(area: GridArea, orientation: Axis) -> Self {
        Self { area, orientation }
    }
}

#[derive(Event, Debug)]
pub struct RoadDestroyEvent {
    pub entity: Entity,
}

impl RoadDestroyEvent {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
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
pub struct RoadExtendEvent {
    pub entity: Entity,
    pub extension: GridArea,
}

impl RoadExtendEvent {
    pub fn new(entity: Entity, extension: GridArea) -> Self {
        Self { entity, extension }
    }
}

#[derive(Event, Debug)]
pub struct RoadBridgeEvent {
    pub first: Entity,
    pub second: Entity,
}

impl RoadBridgeEvent {
    pub fn new(first: Entity, second: Entity) -> Self {
        Self { first, second }
    }
}
