use bevy::prelude::*;
use bevy_egui::egui::{epaint, Align2};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::save::save_events::SaveRequest;
use crate::{
    schedule::UpdateStage, tools::toolbar::ToolState, tools::toolbar_events::ChangeToolRequest, types::building::*,
    types::intersection::*, types::road_segment::*, types::vehicle::*,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin).init_state::<MouseOver>().add_systems(Startup, ui_theme_selection).add_systems(
            Update,
            (
                update_ui_state.in_set(UpdateStage::UpdateView),
                update_toolbar_window,
                update_stats_window,
            ),
        );
    }
}

#[derive(States, Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseOver {
    #[default]
    Ui,
    World,
}

fn update_ui_state(mut contexts: EguiContexts, mut next_state: ResMut<NextState<MouseOver>>) {
    if let Some(ctx) = contexts.try_ctx_mut() {
        next_state.set(match ctx.is_pointer_over_area() {
            true => MouseOver::Ui,
            false => MouseOver::World,
        });
    };
}

fn ui_theme_selection(mut contexts: EguiContexts) {
    catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MACCHIATO);

    let old = contexts.ctx_mut().style().visuals.clone();

    contexts.ctx_mut().set_visuals(egui::Visuals {
        window_shadow: epaint::Shadow {
            offset: [0.0, 0.0].into(),
            blur: 0.0,
            spread: 0.0,
            color: catppuccin_egui::MACCHIATO.base,
        },
        window_rounding: 0.0.into(),
        ..old
    });
}

pub fn update_toolbar_window(
    mut contexts: EguiContexts,
    mut change_tool: EventWriter<ChangeToolRequest>,
    mut save: EventWriter<SaveRequest>,
    mut next_state: ResMut<NextState<VehicleSpawnState>>,
    state: Res<State<VehicleSpawnState>>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };

    egui::Window::new("Tools")
        .resizable(false)
        .collapsible(true)
        .default_open(false)
        .anchor(Align2::LEFT_BOTTOM, (0.0, 0.0))
        .constrain(true)
        .movable(false)
        .show(ctx, |ui| {
            let tool_button_size = egui::Vec2::new(100.0, 10.0);

            if ui.add(egui::Button::new("[ F5 ] Save Game").min_size(tool_button_size)).clicked() {
                save.send(SaveRequest);
            }
            ui.add_space(20.0);

            if ui.add(egui::Button::new("[ ` ] View").min_size(tool_button_size)).clicked() {
                change_tool.send(ChangeToolRequest(ToolState::View));
            }

            if ui.add(egui::Button::new("[ 1 ] Building").min_size(tool_button_size)).clicked() {
                change_tool.send(ChangeToolRequest(ToolState::Building));
            }

            if ui.add(egui::Button::new("[ 2 ] Road").min_size(tool_button_size)).clicked() {
                change_tool.send(ChangeToolRequest(ToolState::Road));
            }

            if ui.add(egui::Button::new("[ 3 ] Bulldozer").min_size(tool_button_size)).clicked() {
                change_tool.send(ChangeToolRequest(ToolState::Eraser));
            }
            ui.label("[TAB]: Rotate Tool");
            ui.label("[R/F]: Adjust Tool Size");
            ui.add_space(20.0);

            let spawn_text = match state.get() {
                VehicleSpawnState::On => "[ L ] Spawning (On)",
                VehicleSpawnState::Off => "[ L ] Spawning (Off)",
            };

            if ui.add(egui::Button::new(spawn_text).min_size(tool_button_size)).clicked() {
                next_state.set({
                    match state.get() {
                        VehicleSpawnState::On => VehicleSpawnState::Off,
                        VehicleSpawnState::Off => VehicleSpawnState::On,
                    }
                });
            }
            ui.add_space(20.0);
            ui.label("[Left Mouse]: Use tool");
            ui.label("[Middle Mouse]: Rotate");
            ui.label("[Right Mouse]: Pan");
            ui.label("[Scroll Wheel]: Zoom");
            ui.add_space(20.0);
            ui.label("[Ctrl + Left Mouse]: Rotate");
            ui.label("[Alt + Left Mouse]: Pan");
            ui.add_space(20.0);
            ui.label("[Q/E]: Rotate");
            ui.label("[WASD]: Pan");
            ui.add_space(20.0);
            ui.label("[K/M]: Adjust Sunlight");
        });
}

pub fn update_stats_window(
    mut contexts: EguiContexts,
    building_query: Query<&Building>,
    road_query: Query<&RoadSegment>,
    inter_query: Query<&Intersection>,
    vehicle_query: Query<&Vehicle>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };

    egui::Window::new("Stats")
        .resizable(false)
        .collapsible(true)
        .default_open(false)
        .anchor(Align2::RIGHT_BOTTOM, (0.0, 0.0))
        .constrain(true)
        .movable(false)
        .show(ctx, |ui| {
            ui.label(format!("Buidings: {:?}", building_query.iter().count()));
            ui.label(format!("Road Segments: {:?}", road_query.iter().count()));
            ui.label(format!("Intersections: {:?}", inter_query.iter().count()));
            ui.label(format!("Vehicles: {:?}", vehicle_query.iter().count()));
        });
}
