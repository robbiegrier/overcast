use crate::grid_cell::GridCell;
use bevy::prelude::*;

#[derive(Copy, Clone, Debug)]
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
        let mut min = hover_cell.position.clone();
        let mut max = hover_cell.position.clone();

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
            x: self.max.position.x - self.min.position.x,
            y: self.max.position.y - self.min.position.y,
        }
    }

    pub fn union(&self, other: GridArea) -> GridArea {
        GridArea {
            min: GridCell {
                position: IVec2 {
                    x: self.min.position.x.min(other.min.position.x),
                    y: self.min.position.y.min(other.min.position.y),
                },
            },
            max: GridCell {
                position: IVec2 {
                    x: self.max.position.x.max(other.max.position.x),
                    y: self.max.position.y.max(other.max.position.y),
                },
            },
        }
    }

    pub fn adjacent_bottom(&self) -> GridArea {
        let new_y = self.min.position.y - 1;
        GridArea {
            min: GridCell::new(self.min.position.x, new_y),
            max: GridCell::new(self.max.position.x, new_y),
        }
    }

    pub fn adjacent_top(&self) -> GridArea {
        let new_y = self.max.position.y + 1;
        GridArea {
            min: GridCell::new(self.min.position.x, new_y),
            max: GridCell::new(self.max.position.x, new_y),
        }
    }

    pub fn adjacent_left(&self) -> GridArea {
        let new_x = self.min.position.x - 1;
        GridArea {
            min: GridCell::new(new_x, self.min.position.y),
            max: GridCell::new(new_x, self.max.position.y),
        }
    }

    pub fn adjacent_right(&self) -> GridArea {
        let new_x = self.max.position.x + 1;
        GridArea {
            min: GridCell::new(new_x, self.min.position.y),
            max: GridCell::new(new_x, self.max.position.y),
        }
    }

    pub fn iter(&self) -> GridAreaIterator {
        GridAreaIterator {
            area: self,
            current: GridCell::new(self.min.position.x - 1, self.min.position.y),
        }
    }
}

pub struct GridAreaIterator<'a> {
    area: &'a GridArea,
    current: GridCell,
}

impl<'a> Iterator for GridAreaIterator<'a> {
    type Item = GridCell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.position.x < self.area.max.position.x {
            self.current = GridCell::new(self.current.position.x + 1, self.current.position.y);
            Some(self.current)
        } else if self.current.position.y < self.area.max.position.y {
            self.current = GridCell::new(self.area.min.position.x, self.current.position.y + 1);
            Some(self.current)
        } else {
            None
        }
    }
}
