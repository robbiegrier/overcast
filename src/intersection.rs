use crate::grid_area::GridArea;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Intersection {
    pub area: GridArea,
    pub roads: [Option<Entity>; 4],
}

impl Intersection {
    pub fn new(area: GridArea) -> Self {
        Self { area, roads: [None; 4] }
    }

    pub fn area(&self) -> GridArea {
        self.area
    }

    pub fn pos(&self) -> Vec3 {
        self.area.center()
    }
}
