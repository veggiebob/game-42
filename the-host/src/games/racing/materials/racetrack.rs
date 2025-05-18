use bevy::asset::{Asset, Handle};
use bevy::prelude::{AlphaMode, TypePath};
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError};
use bevy::color::LinearRgba;
use bevy::image::Image;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::render::mesh::MeshVertexBufferLayoutRef;

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct RacetrackMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

const GROUND_VERT_PATH: &str = "shaders/racing/racetrack.vert";
const GROUND_FRAG_PATH: &str = "shaders/racing/racetrack.frag";

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
/// When using the GLSL shading language for your shader, the specialize method must be overridden.
impl Material for RacetrackMaterial {
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