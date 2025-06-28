use bevy::asset::Assets;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Color, Component, Handle, Res, ResMut};

pub const CAR_BODY_MAT_NAME: &str = "body";

#[derive(Component, Debug)]
pub struct CarStyle {
    pub color: Color,
    pub handle: Handle<StandardMaterial>,
}

impl CarStyle {
    pub fn new(color: Color) -> Self {
        CarStyle {
            color,
            handle: Handle::default(),
        }
    }
    pub fn set_material(&mut self, material: StandardMaterial, material_assets: &mut Assets<StandardMaterial>) {
        self.handle = material_assets.add(material);
        self.apply_style(material_assets);
    }
    
    pub fn apply_style(&self, material_assets: &mut Assets<StandardMaterial>) {
        if let Some(material) = material_assets.get_mut(self.handle.id()) {
            material.base_color = self.color;
        }
    }
}