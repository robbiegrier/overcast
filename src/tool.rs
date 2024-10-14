use crate::{building_tool::BuildingToolPlugin, road_tool::RoadToolPlugin};
use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolState {
    Building,
    #[default]
    Road,
    View,
}

pub struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ToolState>().add_systems(Update, change_tool).add_plugins((BuildingToolPlugin, RoadToolPlugin));
    }
}

pub fn change_tool(mut next_state: ResMut<NextState<ToolState>>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        println!("Entering building tool");
        next_state.set(ToolState::Building);
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        println!("Entering road tool");
        next_state.set(ToolState::Road);
    } else if keyboard_input.just_pressed(KeyCode::Backquote) {
        println!("Entering view tool");
        next_state.set(ToolState::View);
    }
}
