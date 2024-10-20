use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct OnRoadSpawned(pub Entity);

#[derive(Event, Debug)]
pub struct OnIntersectionSpawned(pub Entity);

#[derive(Event, Debug)]
pub struct OnBuildingSpawned(pub Entity);

#[derive(Event, Debug)]
pub struct OnRoadDestroyed(pub Entity);
