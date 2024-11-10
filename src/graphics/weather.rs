use crate::schedule::UpdateStage;
use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_lights).add_systems(Update, adjust_weather.in_set(UpdateStage::UserInput));
    }
}

fn spawn_lights(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 3,
            maximum_distance: 100.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn adjust_weather(mut dir_light_query: Query<&mut DirectionalLight>, keyboard: Res<ButtonInput<KeyCode>>) {
    for mut light in &mut dir_light_query {
        if keyboard.just_pressed(KeyCode::KeyK) {
            light.illuminance += 1_000.0;
        } else if keyboard.just_pressed(KeyCode::KeyM) {
            light.illuminance -= 1_000.0;
        }
    }
}
