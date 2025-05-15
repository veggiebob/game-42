//! Asset management
//! Was going to use this to hot-reload assets, but Bevy already does that.
//! So, this is unused. However, we might want to use a similar pattern for
//! reading config files to manage our game, so I'll keep it around for now.

use bevy::asset::{Asset, AssetServer};
use bevy::log::{error, info};
use bevy::prelude::{Bundle, Commands, Component, Entity, Query, Res, Resource, With};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Component)]
pub struct DoReloadAssets;

pub fn update_reloadable_assets(
    mut commands: Commands,
    asset_entity: Query<(Entity, &Reload)>,
    asset_spawners: Query<&mut Reloader>,
    asset_server: Res<AssetServer>,
    ready_update: Query<Entity, With<DoReloadAssets>>,
) {
    let mut do_reload = false;
    for e in ready_update {
        do_reload = true;
        commands.entity(e).despawn();
    }
    if do_reload {
        reload_reloadable_assets(commands, asset_entity, asset_spawners, asset_server)
    }
}

pub fn reload_reloadable_assets(
    mut commands: Commands,
    asset_entity: Query<(Entity, &Reload)>,
    asset_spawners: Query<&mut Reloader>,
    asset_server: Res<AssetServer>,
) {
    let mut reloaded = HashSet::new();
    for (ent, id) in asset_entity {
        commands.entity(ent).despawn();
        reloaded.insert(id.id);
    }
    let mut num_reloaded = 0;
    for mut asset_spawner in asset_spawners {
        if !asset_spawner.loaded || reloaded.contains(&asset_spawner.id.id) {
            if let Err(_e) = asset_spawner.load.lock().map(|spawner| {
                spawner(&mut commands, asset_server.as_ref());
            }) {
                error!("Unable to reload {}", asset_spawner.id.name);
            } else {
                let sp = asset_spawner.as_mut();
                sp.loaded = true;
                num_reloaded += 1;
                info!("Reloaded {}", asset_spawner.id.name);
            }
        }
    }
    info!("Reloaded {num_reloaded} assets!");
}

#[derive(Resource)]
pub struct ReloadManager {
    next_id: u64,
}

#[derive(Component, Clone)]
pub struct Reload {
    name: String,
    id: u64,
}

#[derive(Component)]
pub struct Reloader {
    id: Reload,
    loaded: bool,
    load: Arc<Mutex<dyn Fn(&mut Commands, &AssetServer) + Send + 'static>>,
}

impl ReloadManager {
    pub fn new() -> Self {
        ReloadManager { next_id: 0 }
    }
    pub fn make_reloader<B, F: Fn(&AssetServer) -> B + Send + 'static>(
        &mut self,
        name: &str,
        get_bundle: F,
    ) -> Reloader
    where
        B: Bundle,
    {
        let id = self.next_id;
        self.next_id += 1;
        Reloader::new(
            Reload {
                name: name.to_string(),
                id,
            },
            get_bundle,
        )
    }
}

impl Reloader {
    fn new<B, F>(id: Reload, get_bundle: F) -> Self
    where
        B: Bundle,
        F: Fn(&AssetServer) -> B + Send + 'static,
    {
        Reloader {
            id: id.clone(),
            loaded: false,
            load: Arc::new(Mutex::new(
                move |commands: &mut Commands, asset_server: &AssetServer| {
                    commands.spawn((get_bundle(asset_server), id.clone()));
                },
            )),
        }
    }
}
