use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct OnRoadSpawned(pub Entity);

impl AsRef<Entity> for OnRoadSpawned {
    fn as_ref(&self) -> &Entity {
        &self.0
    }
}

#[derive(Event, Debug)]
pub struct OnIntersectionSpawned(pub Entity);

impl AsRef<Entity> for OnIntersectionSpawned {
    fn as_ref(&self) -> &Entity {
        &self.0
    }
}

#[derive(Event, Debug)]
pub struct OnBuildingSpawned(pub Entity);

impl AsRef<Entity> for OnBuildingSpawned {
    fn as_ref(&self) -> &Entity {
        &self.0
    }
}

#[derive(Event, Debug)]
pub struct OnRoadDestroyed(pub Entity);

impl AsRef<Entity> for OnRoadDestroyed {
    fn as_ref(&self) -> &Entity {
        &self.0
    }
}

#[derive(Event, Debug)]
pub struct OnIntersectionDestroyed(pub Entity);

impl AsRef<Entity> for OnIntersectionDestroyed {
    fn as_ref(&self) -> &Entity {
        &self.0
    }
}

#[derive(Event, Debug)]
pub struct OnBuildingDestroyed(pub Entity);

impl AsRef<Entity> for OnBuildingDestroyed {
    fn as_ref(&self) -> &Entity {
        &self.0
    }
}
