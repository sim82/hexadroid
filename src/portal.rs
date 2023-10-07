use bevy::prelude::*;

use crate::{
    tiles::{TileCache, TilePos, TileType, TilesState},
    Despawn,
};

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

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(portal_toggle_system);
    }
}
