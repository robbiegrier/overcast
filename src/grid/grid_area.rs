use crate::grid::{grid_cell::*, orientation::*};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct GridArea {
    pub min: GridCell,
    pub max: GridCell,
}

impl GridArea {
    pub fn new(min: GridCell, max: GridCell) -> Self {
        Self { min, max }
    }

    pub fn at(location: Vec3, width: i32, height: i32) -> Self {
        let hover_cell = GridCell::at(location);
        let mut min = hover_cell.pos.clone();
        let mut max = hover_cell.pos.clone();

        if width % 2 != 0 {
            let radius = (width - 1) / 2;
            min.x -= radius;
            max.x += radius;
        } else {
            let radius = width / 2;
            min.x -= radius - 1;
            max.x += radius;
        }

        if height % 2 != 0 {
            let radius = (height - 1) / 2;
            min.y -= radius;
            max.y += radius;
        } else {
            let radius = height / 2;
            min.y -= radius - 1;
            max.y += radius;
        }

        Self {
            min: GridCell::new(min.x, min.y),
            max: GridCell::new(max.x, max.y),
        }
    }

    pub fn center(&self) -> Vec3 {
        let center_2d = (self.min.min_corner() + self.max.max_corner()) / 2.0;
        Vec3::new(center_2d.x, 0.0, center_2d.z)
    }

    pub fn dimensions(&self) -> Vec2 {
        let max = self.max.max_corner();
        let min = self.min.min_corner();
        Vec2 {
            x: max.x - min.x,
            y: max.z - min.z,
        }
    }

    pub fn cell_dimensions(&self) -> IVec2 {
        IVec2 {
            x: (self.max.pos.x - self.min.pos.x) + 1,
            y: (self.max.pos.y - self.min.pos.y) + 1,
        }
    }

    pub fn contains_point_3d(&self, point: Vec3) -> bool {
        self.min.min_corner().x <= point.x
            && self.max.max_corner().x >= point.x
            && self.min.min_corner().z <= point.z
            && self.max.max_corner().z >= point.z
    }

    pub fn union(&self, other: GridArea) -> GridArea {
        GridArea {
            min: GridCell {
                pos: IVec2 {
                    x: self.min.pos.x.min(other.min.pos.x),
                    y: self.min.pos.y.min(other.min.pos.y),
                },
            },
            max: GridCell {
                pos: IVec2 {
                    x: self.max.pos.x.max(other.max.pos.x),
                    y: self.max.pos.y.max(other.max.pos.y),
                },
            },
        }
    }

    pub fn adjacent_bottom(&self) -> GridArea {
        let new_y = self.min.pos.y - 1;
        GridArea {
            min: GridCell::new(self.min.pos.x, new_y),
            max: GridCell::new(self.max.pos.x, new_y),
        }
    }

    pub fn adjacent_top(&self) -> GridArea {
        let new_y = self.max.pos.y + 1;
        GridArea {
            min: GridCell::new(self.min.pos.x, new_y),
            max: GridCell::new(self.max.pos.x, new_y),
        }
    }

    pub fn adjacent_left(&self) -> GridArea {
        let new_x = self.min.pos.x - 1;
        GridArea {
            min: GridCell::new(new_x, self.min.pos.y),
            max: GridCell::new(new_x, self.max.pos.y),
        }
    }

    pub fn adjacent_right(&self) -> GridArea {
        let new_x = self.max.pos.x + 1;
        GridArea {
            min: GridCell::new(new_x, self.min.pos.y),
            max: GridCell::new(new_x, self.max.pos.y),
        }
    }

    pub fn iter(&self) -> GridAreaIterator {
        GridAreaIterator {
            area: self,
            current: GridCell::new(self.min.pos.x - 1, self.min.pos.y),
        }
    }

    pub fn adjacent_areas(&self) -> GridAdjacentAreasIterator {
        GridAdjacentAreasIterator { area: self, index: 0 }
    }
}

pub struct GridAreaIterator<'a> {
    area: &'a GridArea,
    current: GridCell,
}

impl<'a> Iterator for GridAreaIterator<'a> {
    type Item = GridCell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.pos.x < self.area.max.pos.x {
            self.current = GridCell::new(self.current.pos.x + 1, self.current.pos.y);
            Some(self.current)
        } else if self.current.pos.y < self.area.max.pos.y {
            self.current = GridCell::new(self.area.min.pos.x, self.current.pos.y + 1);
            Some(self.current)
        } else {
            None
        }
    }
}

pub struct GridAdjacentAreasIterator<'a> {
    area: &'a GridArea,
    index: usize,
}

impl<'a> Iterator for GridAdjacentAreasIterator<'a> {
    type Item = (GridArea, GDir);

    fn next(&mut self) -> Option<Self::Item> {
        let next = match self.index {
            0 => Some((self.area.adjacent_top(), GDir::North)),
            1 => Some((self.area.adjacent_bottom(), GDir::South)),
            2 => Some((self.area.adjacent_left(), GDir::West)),
            3 => Some((self.area.adjacent_right(), GDir::East)),
            _ => None,
        };
        self.index += 1;
        next
    }
}
