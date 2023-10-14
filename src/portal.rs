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

#[derive(Component)]
pub struct PortalToggleRequest;

pub fn portal_toggle_system_2(
    mut commands: Commands,
    tiles_state: Res<TilesState>,
    mut tile_cache: ResMut<TileCache>,
    query: Query<(Entity, &TilePos), With<PortalToggleRequest>>,
) {
    for (portal_entity, portal_pos) in &query {
        if let Some(entity) = tile_cache.tiles.remove(portal_pos) {
            // info!("delete");
            commands.entity(entity).insert(Despawn::ThisFrame);
        } else {
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
        commands.entity(portal_entity).despawn();
    }
}
pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (portal_toggle_system, portal_toggle_system_2));
    }
}
