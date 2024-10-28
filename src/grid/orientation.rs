use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum GAxis {
    #[default]
    X,
    Z,
}

impl GAxis {}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GDir {
    North,
    South,
    West,
    East,
}

impl GDir {
    pub fn inverse(&self) -> GDir {
        match *self {
            GDir::North => GDir::South,
            GDir::South => GDir::North,
            GDir::West => GDir::East,
            GDir::East => GDir::West,
        }
    }

    pub fn index(&self) -> usize {
        match &self {
            GDir::North => 0,
            GDir::South => 1,
            GDir::West => 2,
            GDir::East => 3,
        }
    }

    pub fn binary_index(&self) -> usize {
        match &self {
            GDir::North => 0,
            GDir::South => 1,
            GDir::West => 0,
            GDir::East => 1,
        }
    }

    pub fn as_dir3(&self) -> Dir3 {
        match &self {
            GDir::North => Dir3::Z,
            GDir::South => Dir3::NEG_Z,
            GDir::West => Dir3::X,
            GDir::East => Dir3::NEG_X,
        }
    }

    pub fn as_vec3(&self) -> Vec3 {
        self.as_dir3().as_vec3()
    }
}
