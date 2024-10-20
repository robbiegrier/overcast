use bevy::prelude::*;

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                UpdateStage::UpdateView,
                UpdateStage::UserInput,
                UpdateStage::HighLevelSideEffects,
                UpdateStage::LowLevelDestruction,
                UpdateStage::LowLevelSpawning,
                UpdateStage::UpdateGraph,
                UpdateStage::Visualize,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (apply_deferred).after(UpdateStage::LowLevelSpawning).before(UpdateStage::UpdateGraph),
        );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum UpdateStage {
    UpdateView,
    UserInput,
    HighLevelSideEffects,
    LowLevelDestruction,
    LowLevelSpawning,
    UpdateGraph,
    Visualize,
}
