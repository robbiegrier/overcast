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
}
