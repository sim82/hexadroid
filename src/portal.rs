use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Portal {
    pub timer: Timer,
    pub tile_pos: TilePos,
}

pub fn portal_toggle_system(
    mut commands: Commands,
    time: Res<Time>,
    tiles_state: Res<TilesState>,
    mut tile_cache: ResMut<TileCache>,
    mut query: Query<&mut Portal>,
) {
    for mut portal in &mut query {
        portal.timer.tick(time.delta());
        if portal.timer.just_finished() {
            if let Some(entity) = tile_cache.tiles.remove(&portal.tile_pos) {
                // info!("delete");
                commands.entity(entity).insert(Despawn::ThisFrame);
            } else {
                let entity = commands
                    .spawn(SpatialBundle::default())
                    .insert(TileType {
                        wall: true,
                        immediate_collider: true,
                    })
                    .insert(portal.tile_pos)
                    .id();
                commands.entity(tiles_state.tile_root).add_child(entity);
            }
        }
    }
}

#[derive(Component, Default)]
pub struct PortalToggleRequest {
    boundary_only: bool,
}

impl PortalToggleRequest {
    pub(crate) fn boundary_only() -> Self {
        Self {
            boundary_only: true,
        }
    }
}

pub fn portal_toggle_system_2(
    mut commands: Commands,
    tiles_state: Res<TilesState>,
    mut tile_cache: ResMut<TileCache>,
    query: Query<(Entity, &TilePos, &PortalToggleRequest)>,
) {
    for (portal_entity, portal_pos, toggle_request) in &query {
        let tile_exists = tile_cache.tiles.contains_key(portal_pos);
        if tile_exists {
            if !toggle_request.boundary_only
                || portal_pos
                    .get_neighbors()
                    .iter()
                    .any(|np| !tile_cache.tiles.contains_key(np))
            {
                let entity = tile_cache
                    .tiles
                    .remove(portal_pos)
                    .expect("tile remove failed, despite check"); // cannot fail because we explicitly check before
                commands.entity(entity).insert(Despawn::ThisFrame);
            }
        } else {
            if !toggle_request.boundary_only
                || portal_pos
                    .get_neighbors()
                    .iter()
                    .any(|np| tile_cache.tiles.contains_key(np))
            {
                let entity = commands
                    .spawn(SpatialBundle::default())
                    .insert(TileType {
                        wall: true,
                        immediate_collider: true,
                    })
                    .insert(*portal_pos)
                    .id();
                commands.entity(tiles_state.tile_root).add_child(entity);
            }
        }
        // match tile_cache.tiles.entry(*portal_pos) {
        //     bevy::utils::hashbrown::hash_map::Entry::Occupied(ent) => {
        //         commands.entity(*ent.get()).insert(Despawn::ThisFrame);
        //         ent.remove_entry();
        //     }
        //     bevy::utils::hashbrown::hash_map::Entry::Vacant(ent) => {
        //         let entity = commands
        //             .spawn(SpatialBundle::default())
        //             .insert(TileType {
        //                 wall: true,
        //                 immediate_collider: true,
        //             })
        //             .insert(*portal_pos)
        //             .id();
        //         commands.entity(tiles_state.tile_root).add_child(entity);
        //     }
        // }
        // if let Some(entity) = tile_cache.tiles.remove(portal_pos) {
        //     // info!("delete");
        //     commands.entity(entity).insert(Despawn::ThisFrame);
        // } else {
        //     let entity = commands
        //         .spawn(SpatialBundle::default())
        //         .insert(TileType {
        //             wall: true,
        //             immediate_collider: true,
        //         })
        //         .insert(*portal_pos)
        //         .id();
        //     commands.entity(tiles_state.tile_root).add_child(entity);
        // }
        commands.entity(portal_entity).despawn();
    }
}
pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (portal_toggle_system, portal_toggle_system_2));
    }
}
