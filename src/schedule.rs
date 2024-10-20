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
                UpdateStage::SoftDestroy,
                UpdateStage::Spawning,
                UpdateStage::AfterSpawning,
                UpdateStage::Analyze,
                UpdateStage::DestroyEntities,
                UpdateStage::Visualize,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (apply_deferred).after(UpdateStage::Spawning).before(UpdateStage::AfterSpawning),
        );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum UpdateStage {
    UpdateView,
    UserInput,
    HighLevelSideEffects,
    SoftDestroy,
    Spawning,
    AfterSpawning,
    Analyze,
    DestroyEntities,
    Visualize,
}
