use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct GridCell {
    pub position: IVec2,
}

impl GridCell {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            position: IVec2::new(x, y),
        }
    }

    pub fn at(location: Vec3) -> Self {
        let position = IVec2::new(location.x.floor() as i32, location.z.floor() as i32);
        Self { position }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(self.position.x as f32 + 0.5, 0.0, self.position.y as f32 + 0.5)
    }

    pub fn max_corner(&self) -> Vec3 {
        Vec3::new(self.position.x as f32 + 1.0, 0.0, self.position.y as f32 + 1.0)
    }

    pub fn min_corner(&self) -> Vec3 {
        Vec3::new(self.position.x as f32, 0.0, self.position.y as f32)
    }
}
