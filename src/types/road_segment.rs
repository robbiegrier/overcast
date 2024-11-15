use crate::{grid::grid_area::*, grid::grid_cell::*, grid::orientation::*};
use bevy::prelude::*;
use bevy::utils::HashSet;

const LANE_MEDIAN_SIZE: f32 = 0.5;
const LANE_CURB: f32 = 0.5;

#[derive(Component, Debug)]
pub struct RoadSegment {
    pub orientation: GAxis,
    pub area: GridArea,
    pub ends: [Option<Entity>; 2],
    pub dests: HashSet<Entity>,
    pub observers: HashSet<Entity>,
}

impl RoadSegment {
    pub fn new(area: GridArea, orientation: GAxis) -> Self {
        Self {
            orientation,
            area,
            ends: [None; 2],
            dests: HashSet::new(),
            observers: HashSet::new(),
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

    pub fn speed_limit(&self) -> f32 {
        self.drive_width() as f32 * 0.25
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

        let lanesf = self.num_lanes() as f32 - 1.0;
        let lane_ind = (num) as f32;
        let curbf = LANE_CURB;
        let medianf = LANE_MEDIAN_SIZE;

        let dir_width = ((lanesf + 1.0) - medianf) - curbf;
        let t = if lanesf == 0.0 { 0.0 } else { lane_ind / lanesf };

        if self.orientation == GAxis::Z {
            if dir == GDir::North {
                let a = cmin.x + curbf;
                let b = a + dir_width;
                let desired = a.lerp(b, t);
                pos.with_x(desired).with_z(pos.z.clamp(cmin.z, cmax.z))
            } else {
                let a = cmax.x - curbf;
                let b = a - dir_width;
                let desired = a.lerp(b, t);
                pos.with_x(desired).with_z(pos.z.clamp(cmin.z, cmax.z))
            }
        } else {
            if dir == GDir::East {
                let a = cmin.z + curbf;
                let b = a + dir_width;
                let desired = a.lerp(b, t);
                pos.with_z(desired).with_x(pos.x.clamp(cmin.x, cmax.x))
            } else {
                let a = cmax.z - curbf;
                let b = a - dir_width;
                let desired = a.lerp(b, t);
                pos.with_z(desired).with_x(pos.x.clamp(cmin.x, cmax.x))
            }
        }
    }
}
