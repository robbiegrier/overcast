use crate::{grid::grid_area::*, grid::grid_cell::*, grid::orientation::*};
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

    #[allow(dead_code)]
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

    pub fn num_lanes(&self) -> i32 {
        self.drive_width() / 2
    }

    pub fn get_intersection_area(&self, turn_to_area: GridArea) -> GridArea {
        match self.orientation {
            GAxis::Z => GridArea::new(
                GridCell::new(self.area.min.pos.x, turn_to_area.min.pos.y),
                GridCell::new(self.area.max.pos.x, turn_to_area.max.pos.y),
            ),
            GAxis::X => GridArea::new(
                GridCell::new(turn_to_area.min.pos.x, self.area.min.pos.y),
                GridCell::new(turn_to_area.max.pos.x, self.area.max.pos.y),
            ),
        }
    }

    pub fn get_lane_pos(&self, start_pos: Vec3) -> Vec3 {
        match self.orientation {
            GAxis::Z => start_pos.with_x(self.area.center().x),
            GAxis::X => start_pos.with_z(self.area.center().z),
        }
    }

    pub fn clamp_to_lane(&self, dir: GDir, num: i32, pos: Vec3) -> Vec3 {
        let cmax = self.area.max.max_corner();
        let cmin = self.area.min.min_corner();
        let lane_id = (num + 1).clamp(1, self.num_lanes());
        let lane_stride = 0.5 * self.num_lanes() as f32 * lane_id as f32;

        if self.orientation == GAxis::Z {
            if dir == GDir::North {
                pos.with_x(cmin.x + lane_stride).with_z(pos.z.clamp(cmin.z, cmax.z))
            } else {
                pos.with_x(cmax.x - lane_stride).with_z(pos.z.clamp(cmin.z, cmax.z))
            }
        } else {
            if dir == GDir::East {
                pos.with_z(cmin.z + lane_stride).with_x(pos.x.clamp(cmin.x, cmax.x))
            } else {
                pos.with_z(cmax.z - lane_stride).with_x(pos.x.clamp(cmin.x, cmax.x))
            }
        }
    }
}
