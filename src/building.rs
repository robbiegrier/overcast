use bevy::prelude::*;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component, Debug)]
pub struct Building;
