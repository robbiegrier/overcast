use crate::grid::grid_area::*;
use bevy::{prelude::*, utils::HashSet};

#[derive(Component, Debug)]
pub struct Intersection {
    pub area: GridArea,
    pub roads: [Option<Entity>; 4],
    pub observers: HashSet<Entity>,
}

impl Intersection {
    pub fn new(area: GridArea) -> Self {
        Self {
            area,
            roads: [None; 4],
            observers: HashSet::new(),
        }
    }

    pub fn area(&self) -> GridArea {
        self.area
    }

    pub fn pos(&self) -> Vec3 {
        self.area.center()
    }
}
