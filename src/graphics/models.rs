use bevy::prelude::*;

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Models::new());
    }
}

pub struct VehicleModelData {
    pub mesh: String,
    pub material: String,
    pub scale: f32,
    pub vertical_offset: f32,
}

impl VehicleModelData {
    pub fn from_voxcar(i: i32, scale: f32, vertical_offset: f32) -> Self {
        VehicleModelData {
            mesh: format!("models/voxcar-{:?}.gltf#Mesh0/Primitive0", i),
            material: format!("models/voxcar-{:?}.gltf#Material0", i),
            scale,
            vertical_offset,
        }
    }
}

#[derive(Resource)]
pub struct Models {
    pub vehicle_models: Vec<VehicleModelData>,
}

impl Models {
    pub fn new() -> Self {
        Models {
            vehicle_models: vec![
                VehicleModelData::from_voxcar(1, 1.0, 0.0),
                VehicleModelData::from_voxcar(2, 1.0, 0.0),
                VehicleModelData::from_voxcar(3, 1.5, 0.2),
                VehicleModelData::from_voxcar(4, 1.2, 0.01),
                VehicleModelData::from_voxcar(5, 1.0, 0.0),
            ],
        }
    }
}
