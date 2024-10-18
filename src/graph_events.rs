use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct GraphEdgeAddEvent(pub Entity);

#[derive(Event, Debug)]
pub struct GraphNodeAddEvent(pub Entity);

#[derive(Event, Debug)]
pub struct GraphDestinationAddEvent(pub Entity);

#[derive(Event, Debug)]
pub struct GraphEdgeRemoveEvent(pub Entity);

#[derive(Event, Debug)]
pub struct GraphNodeRemoveEvent(pub Entity);

#[derive(Event, Debug)]
pub struct GraphDestinationRepairEvent(pub Entity);

#[derive(Event, Debug)]
pub struct GraphEdgeRepairEvent(pub Entity);

#[derive(Event, Debug)]
pub struct GraphNodeRepairEvent(pub Entity);
