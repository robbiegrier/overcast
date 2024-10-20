use crate::{grid_area::GridArea, orientation::GAxis};
use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct RequestRoad {
    pub area: GridArea,
    pub orientation: GAxis,
}

impl RequestRoad {
    pub fn new(area: GridArea, orientation: GAxis) -> Self {
        Self { area, orientation }
    }
}

#[derive(Event, Debug)]
pub struct RequestDeleteRoad {
    pub entity: Entity,
}

impl RequestDeleteRoad {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

#[derive(Event, Debug)]
pub struct RequestIntersection {
    pub area: GridArea,
}

impl RequestIntersection {
    pub fn new(area: GridArea) -> Self {
        Self { area }
    }
}

#[derive(Event, Debug)]
pub struct RequestRoadSplit {
    pub entity: Entity,
    pub split_area: GridArea,
}

impl RequestRoadSplit {
    pub fn new(entity: Entity, split_area: GridArea) -> Self {
        Self { entity, split_area }
    }
}

#[derive(Event, Debug)]
pub struct RequestRoadExtend {
    pub entity: Entity,
    pub extension: GridArea,
}

impl RequestRoadExtend {
    pub fn new(entity: Entity, extension: GridArea) -> Self {
        Self { entity, extension }
    }
}

#[derive(Event, Debug)]
pub struct RequestRoadBridge {
    pub first: Entity,
    pub second: Entity,
}

impl RequestRoadBridge {
    pub fn new(first: Entity, second: Entity) -> Self {
        Self { first, second }
    }
}
