use crate::tools::toolbar::ToolState;
use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct ChangeToolRequest(pub ToolState);
