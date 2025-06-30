use crate::games::racing::style::{CarStyle};
use crate::games::racing::track::{Tragnet, TragnetAnchor};
use crate::games::racing::{COLLISION_MAT_NAME, RACE_CHECKPOINTS, RaceGameMarker, SceneInfo, TRAGNET_MAT_NAME, RacingSceneMarker, CAR_BODY_MAT_NAME};
use avian3d::prelude::{Collider, RigidBody};
use bevy::asset::Assets;
use bevy::gltf::GltfMaterialName;
use bevy::log::{info, warn};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Children, Commands, Mesh, Mesh3d, MeshMaterial3d, Name, NextState, Query, Res, ResMut, Transform, TransformHelper, Trigger};
use bevy::scene::SceneInstanceReady;
use regex::Regex;
use std::collections::HashMap;
use crate::games::GamePhase;
// Steps for updating/exporting the scene:
// 1. Save .blend file in assets/blender/<filename>
// 2. Export to assets/gltf/<asset name>/<asset name>.glb
//      Include:
//          Limit to:
//              ✅ Visible Objects
//      Transform:
//          ✅ +Y Up
//      Data:
//          Mesh:
//              ✅ Apply Modifiers

// Steps for updating the *TRACK*
// 1. Update Bézier curves as desired, to your liking
//      Note: the curve may be hidden
// 2. Go to outliner, search for "tragnet." ("." is important)
// 3. Select all, delete.
// 4. Select "tragnet", duplicate with Shift + D
// 4a. Ensure that it has the material "tragnet"
// 5. Apply all modifiers.
// 6. Tab to Edit Mode.
// 7. Select all.
// 8. Separate by loose parts (P > L).
// 9. Tab back to Object Mode.
// A. With everything selected, Set Origin to Center (Surface) (under Object)
// B. Hide the "tragnet" object.
// C. Export with instructions above.

/// This runs whenever a GLTF asset is loaded, such as the car or the track
pub fn on_scene_load(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    mut car_style: Query<&mut CarStyle>,
    mut next_state: ResMut<NextState<GamePhase>>,
    gltf_children: Query<(&GltfMaterialName, &Mesh3d, &Name)>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    meshes: Res<Assets<Mesh>>,
    children: Query<&Children>,
    transform_helper: TransformHelper,
    mut scene_info: ResMut<SceneInfo>,
    racing_scene_marker: Query<&RacingSceneMarker>,
) {
    let mut current_style = car_style.get_mut(trigger.target()).ok();
    info!("Scene Instance Ready: {:?}", trigger.target());
    let re = Regex::new(r"\d+").ok().unwrap();
    let get_index_from_name = |name| re.find(name).and_then(|m| m.as_str().parse::<usize>().ok());
    let mut anchors = HashMap::new();
    for descendant in children
        .iter_descendants(trigger.target())
        .collect::<Vec<_>>()
        .into_iter()
    {
        if let Ok((gltf_name, mesh, name)) = gltf_children.get(descendant) {
            // add collider to it
            if gltf_name.0 == COLLISION_MAT_NAME {
                // make it into a collider
                if let Some(mesh) = meshes.get(&mesh.0) {
                    if let Some(collider) = Collider::convex_hull_from_mesh(mesh) {
                        commands
                            .entity(descendant)
                            .insert(RigidBody::Static)
                            .remove::<Mesh3d>()
                            .insert(collider);
                    } else {
                        warn!("Unable to generate collider for {name}!",)
                    }
                }
            } else if gltf_name.0 == TRAGNET_MAT_NAME {
                // add it to the tragnet
                let index = get_index_from_name(name.as_str()).unwrap_or(0);
                let transform = transform_helper
                    .compute_global_transform(descendant)
                    .unwrap()
                    .compute_transform();
                anchors.insert(
                    index,
                    TragnetAnchor {
                        transform: Transform::from_translation(transform.translation),
                    },
                );
                commands.entity(descendant).remove::<Mesh3d>();
                // info!("Adding to tragnet. Name is {}. Index {index} and transform {:?}", name.as_str(), transform);
            } else if gltf_name.0 == CAR_BODY_MAT_NAME {
                if let Some(style) = &mut current_style {
                    let material = mesh_materials
                        .get(descendant)
                        .ok()
                        .and_then(|m_mat| material_assets.get(m_mat.id()))
                        .cloned()
                        .unwrap_or(StandardMaterial::default());
                    let handle = material_assets.add(material);
                    style.handle = handle.clone();
                    style.apply_style(material_assets.as_mut());
                    commands.entity(descendant).insert(MeshMaterial3d(handle));
                }
            }
        }
    }
    if racing_scene_marker.get(trigger.target()).is_ok() {
        let mut pts: Vec<_> = anchors.into_iter().collect();
        pts.sort_by_key(|(i, _a)| *i);
        let scene_info = scene_info.as_mut();
        scene_info.race_start = pts[0].1.transform;
        let new_tragnet =
            Tragnet::new(pts.into_iter().map(|(_i, a)| a).collect(), RACE_CHECKPOINTS);
        commands.spawn((RaceGameMarker, new_tragnet));
        // start the game
        // next_state.set(GamePhase::PlayingGame);
    }
}
