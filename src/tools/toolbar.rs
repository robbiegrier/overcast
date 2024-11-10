use crate::{
    schedule::UpdateStage,
    tools::{
        building_tool::BuildingToolPlugin, eraser_tool::EraserToolPlugin, road_tool::RoadToolPlugin, toolbar_events::*,
    },
};
use bevy::prelude::*;

#[derive(States, Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ToolState {
    Building,
    Road,
    Eraser,
    #[default]
    View,
}

pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ToolState>()
            .add_event::<ChangeToolRequest>()
            .add_plugins((BuildingToolPlugin, RoadToolPlugin, EraserToolPlugin))
            .add_systems(
                Update,
                (
                    change_tool_on_keypress.in_set(UpdateStage::UserInput),
                    handle_change_tool_requests,
                )
                    .chain(),
            );
    }
}

pub fn change_tool_on_keypress(keyboard_input: Res<ButtonInput<KeyCode>>, mut change_tool: EventWriter<ChangeToolRequest>) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        change_tool.send(ChangeToolRequest(ToolState::Building));
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        change_tool.send(ChangeToolRequest(ToolState::Road));
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        change_tool.send(ChangeToolRequest(ToolState::Eraser));
    } else if keyboard_input.just_pressed(KeyCode::Backquote) {
        change_tool.send(ChangeToolRequest(ToolState::View));
    }
}

pub fn handle_change_tool_requests(mut event: EventReader<ChangeToolRequest>, mut next_state: ResMut<NextState<ToolState>>) {
    for &ChangeToolRequest(mode) in event.read() {
        next_state.set(mode);
    }
}
