use std::collections::HashMap;
use crate::games::racing::RaceGameMarker;
use crate::games::racing::materials::race_rails::RailsMaterial;
use bevy::gltf::GltfMaterialName;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, StandardMaterial};
use bevy::prelude::{
    AlphaMode, Asset, Assets, Children, Commands, Component, Entity, Handle, Image, LinearRgba,
    Material, MeshMaterial3d, Plugin, Query, ResMut, Trigger, TypePath, With, World, info,
};
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use bevy::scene::SceneInstanceReady;
use racetrack::RacetrackMaterial;

pub mod race_rails;
pub mod racetrack;

#[derive(Debug)]
pub enum MaterialOverride {
    Racetrack(Handle<RacetrackMaterial>),
    Rails(Handle<RacetrackMaterial>),
}

#[derive(Component, Debug)]
pub struct MaterialOverrides(HashMap<String, MaterialOverride>);

impl MaterialOverride {
    pub fn get_gltf_name(&self) -> &str {
        match self {
            MaterialOverride::Racetrack(_) => "road-simple",
            MaterialOverride::Rails(_) => "road-rails",
        }
    }
}

impl From<MaterialOverride> for MaterialOverrides {
    fn from(value: MaterialOverride) -> Self {
        MaterialOverrides::new(vec![value].into_iter())
    }
}

impl MaterialOverrides {
    pub fn new(overrides: impl Iterator<Item = MaterialOverride>) -> MaterialOverrides {
        MaterialOverrides(
            overrides
                .map(|ov| (ov.get_gltf_name().to_string(), ov))
                .collect(),
        )
    }
}

// pub fn material_override(
//     trigger: Trigger<SceneInstanceReady>,
//     mut commands: Commands,
//     mat_override_query: Query<&MaterialOverrides, With<RaceGameMarker>>,
//     gltf_children: Query<&GltfMaterialName, With<MeshMaterial3d<StandardMaterial>>>,
//     children: Query<&Children>,
// ) {
//     info!("Scene Instance Ready: {:?}", trigger.target());
//     let Ok(mat_overrides) = mat_override_query.get(trigger.target()) else {
//         return;
//     };
//     for descendant in children.iter_descendants(trigger.target()).collect::<Vec<_>>().into_iter() {
//         if let Ok(gltf_name) = gltf_children.get(descendant) {
//             info!("Checking {}", gltf_name.0);
//             if let Some(mat_override) = mat_overrides.0.get(&gltf_name.0) {
//                 info!("Replacing material {}", mat_override.get_gltf_name());
//                 match mat_override {
//                     MaterialOverride::Racetrack(mat_handle) => {
//                         commands
//                             .entity(descendant)
//                             .remove::<MeshMaterial3d<StandardMaterial>>();
//                         commands
//                             .entity(descendant)
//                             .insert(MeshMaterial3d(mat_handle.clone()));
//                     }
//                     MaterialOverride::Rails(mat_handle) => {
//                         commands
//                             .entity(descendant)
//                             .remove::<MeshMaterial3d<StandardMaterial>>();
//                         commands
//                             .entity(descendant)
//                             .insert(MeshMaterial3d(mat_handle.clone()));
//                     }
//                     _ => {}
//                 };
//             }
//         }
//     }
// }
