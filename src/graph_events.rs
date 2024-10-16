use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct GraphEdgeAddEvent(pub Entity);

#[derive(Event, Debug)]
pub struct GraphNodeAddEvent(pub Entity);

#[derive(Event, Debug)]
pub struct GraphEdgeRemoveEvent(pub Entity);

#[derive(Event, Debug)]
pub struct GraphNodeRemoveEvent(pub Entity);
