use crate::grid_area::*;
use crate::grid_cell::*;
use crate::road_tool::Axis;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct RoadSegment {
    pub orientation: Axis,
    pub area: GridArea,
}

impl RoadSegment {
    pub fn new(area: GridArea, orientation: Axis) -> Self {
        Self { orientation, area }
    }

    pub fn drive_length(&self) -> i32 {
        match self.orientation {
            Axis::Z => self.area.cell_dimensions().y,
            Axis::X => self.area.cell_dimensions().x,
        }
    }

    pub fn drive_width(&self) -> i32 {
        match self.orientation {
            Axis::Z => self.area.cell_dimensions().x,
            Axis::X => self.area.cell_dimensions().y,
        }
    }

    pub fn get_intersection_area(&self, turn_to_area: GridArea) -> GridArea {
        match self.orientation {
            Axis::Z => GridArea::new(
                GridCell::new(self.area.min.position.x, turn_to_area.min.position.y),
                GridCell::new(self.area.max.position.x, turn_to_area.max.position.y),
            ),
            Axis::X => GridArea::new(
                GridCell::new(turn_to_area.min.position.x, self.area.min.position.y),
                GridCell::new(turn_to_area.max.position.x, self.area.max.position.y),
            ),
        }
    }
}
