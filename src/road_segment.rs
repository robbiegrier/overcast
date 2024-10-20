use crate::grid_area::*;
use crate::grid_cell::*;
use crate::orientation::GAxis;
use bevy::prelude::*;
use bevy::utils::HashSet;

#[derive(Component, Debug)]
pub struct RoadSegment {
    pub orientation: GAxis,
    pub area: GridArea,
    pub ends: [Option<Entity>; 2],
    pub dests: HashSet<Entity>,
}

impl RoadSegment {
    pub fn new(area: GridArea, orientation: GAxis) -> Self {
        Self {
            orientation,
            area,
            ends: [None; 2],
            dests: HashSet::new(),
        }
    }

    pub fn area(&self) -> GridArea {
        self.area
    }

    pub fn pos(&self) -> Vec3 {
        self.area.center()
    }

    pub fn drive_length(&self) -> i32 {
        match self.orientation {
            GAxis::Z => self.area.cell_dimensions().y,
            GAxis::X => self.area.cell_dimensions().x,
        }
    }

    pub fn drive_width(&self) -> i32 {
        match self.orientation {
            GAxis::Z => self.area.cell_dimensions().x,
            GAxis::X => self.area.cell_dimensions().y,
        }
    }

    pub fn get_intersection_area(&self, turn_to_area: GridArea) -> GridArea {
        match self.orientation {
            GAxis::Z => GridArea::new(
                GridCell::new(self.area.min.position.x, turn_to_area.min.position.y),
                GridCell::new(self.area.max.position.x, turn_to_area.max.position.y),
            ),
            GAxis::X => GridArea::new(
                GridCell::new(turn_to_area.min.position.x, self.area.min.position.y),
                GridCell::new(turn_to_area.max.position.x, self.area.max.position.y),
            ),
        }
    }

    pub fn get_lane_pos(&self, start_pos: Vec3) -> Vec3 {
        match self.orientation {
            GAxis::Z => start_pos.with_x(self.area.center().x),
            GAxis::X => start_pos.with_z(self.area.center().z),
        }
    }
}
