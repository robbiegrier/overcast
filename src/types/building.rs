use crate::grid::grid_area::*;
use bevy::{prelude::*, utils::HashSet};

#[derive(Component, Debug)]
pub struct Building {
    pub area: GridArea,
    pub roads: HashSet<Entity>,
}

impl Building {
    pub fn new(area: GridArea) -> Self {
        Self {
            area,
            roads: HashSet::new(),
        }
    }

    pub fn area(&self) -> GridArea {
        self.area
    }

    pub fn pos(&self) -> Vec3 {
        self.area.center()
    }
}
