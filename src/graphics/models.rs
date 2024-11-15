use bevy::prelude::*;

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Models::new()).add_systems(Startup, load_models);
    }
}

pub struct VehicleModelData {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub scale: f32,
    pub vertical_offset: f32,
}

impl VehicleModelData {
    pub fn from_voxcar(i: i32, scale: f32, vertical_offset: f32, asset_server: &Res<AssetServer>) -> Self {
        VehicleModelData {
            mesh: asset_server.load(format!("models/voxcar-{:?}.gltf#Mesh0/Primitive0", i)),
            material: asset_server.load(format!("models/voxcar-{:?}.gltf#Material0", i)),
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
            vehicle_models: Vec::new(),
        }
    }
}

fn load_models(asset_server: Res<AssetServer>, mut models: ResMut<Models>) {
    models.vehicle_models.push(VehicleModelData::from_voxcar(1, 1.0, 0.0, &asset_server));
    models.vehicle_models.push(VehicleModelData::from_voxcar(2, 1.0, 0.0, &asset_server));
    models.vehicle_models.push(VehicleModelData::from_voxcar(3, 1.5, 0.2, &asset_server));
    models.vehicle_models.push(VehicleModelData::from_voxcar(4, 1.2, 0.01, &asset_server));
    models.vehicle_models.push(VehicleModelData::from_voxcar(5, 1.0, 0.0, &asset_server));
}
