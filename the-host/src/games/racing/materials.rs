use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, StandardMaterial};
use bevy::prelude::{info, AlphaMode, Asset, Assets, Children, Commands, Component, Handle, Image, LinearRgba, Material, MeshMaterial3d, Query, ResMut, Trigger, TypePath, With};
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError};
use bevy::scene::SceneInstanceReady;
use rand::distributions::Standard;
use crate::games::racing::RaceGameMarker;

#[derive(Component)]
pub enum MaterialOverride {
    Ground(RacingGroundMaterial),
}

pub fn material_override(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    mat_override: Query<&MaterialOverride, With<RaceGameMarker>>,
    current_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
    children: Query<&Children>,
    mut ground_materials: ResMut<Assets<RacingGroundMaterial>>,
) {
    info!("Scene Instance Ready: {:?}", trigger.target());
    let Ok(mat_override) = mat_override.get(trigger.target()) else {
        return;
    };
    for descendant in children.iter_descendants(trigger.target()) {
        if let Some(material) = current_materials
            .get(descendant)
            .ok()
            .and_then(|id| asset_materials.get_mut(id.id()))
        {
            info!("Replacing material");
            commands.entity(descendant).remove::<MeshMaterial3d<StandardMaterial>>();
            commands.entity(descendant)
                .insert(MeshMaterial3d(
                    match mat_override {
                        MaterialOverride::Ground(rgm) => {
                            ground_materials.add(rgm.clone())
                        }
                    }
                ));
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct RacingGroundMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

const GROUND_VERT_PATH: &str = "shaders/racing/ground.vert";
const GROUND_FRAG_PATH: &str = "shaders/racing/ground.frag";

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
/// When using the GLSL shading language for your shader, the specialize method must be overridden.
impl Material for RacingGroundMaterial {
    fn vertex_shader() -> ShaderRef {
        GROUND_VERT_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        GROUND_FRAG_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    // Bevy assumes by default that vertex shaders use the "vertex" entry point
    // and fragment shaders use the "fragment" entry point (for WGSL shaders).
    // GLSL uses "main" as the entry point, so we must override the defaults here
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();
        Ok(())
    }
}