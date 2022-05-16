use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::prelude::*;
use hexagon_tiles::{hexagon::HexMath, hexagon::HEX_DIRECTIONS, layout::LayoutTool};

use crate::HEX_LAYOUT;

#[derive(Component)]
pub struct TileType {
    pub wall: bool,
}

#[derive(Component, Eq, PartialEq, Copy, Clone)]
pub struct TilePos(pub hexagon_tiles::hexagon::Hex);

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for TilePos {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.q().hash(state);
        self.0.r().hash(state);
        self.0.s().hash(state);
    }
}

#[derive(Default)]
pub struct TileCache {
    pub tiles: HashMap<TilePos, Entity>,
}

fn spawn_tiles_system(
    mut commands: Commands,
    mut tiles_cache: ResMut<TileCache>,
    query: Query<(Entity, &TilePos, &TileType), Added<TileType>>,
) {
    for (entity, tile_pos, _) in query.iter() {
        // commands.entity(entity).inser

        let corners = LayoutTool::polygon_corners(HEX_LAYOUT, tile_pos.0)
            .iter()
            .map(|p| Vec2::new(p.x as f32, p.y as f32))
            .collect();

        commands
            .entity(entity)
            .insert(Collider::polyline(
                corners,
                Some(vec![[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 0]]),
            ))
            .insert(Transform::from_xyz(0.0, 0.0, 0.0))
            .insert(RigidBody::Fixed);

        tiles_cache.tiles.insert(*tile_pos, entity);
    }
}

fn optimize_colliders_system(
    mut commands: Commands,
    time: Res<Time>,
    mut delay: Local<f32>,
    tile_cache: ResMut<TileCache>,
    query: Query<(Entity, &TilePos, &TileType)>,
) {
    if *delay > 0.0 {
        *delay -= time.delta_seconds();
        return;
    }
    *delay = 0.5;
    for (entity, tile_pos, _) in query.iter() {
        // commands.entity(entity).inser

        let corners = LayoutTool::polygon_corners(HEX_LAYOUT, tile_pos.0)
            .iter()
            .map(|p| Vec2::new(p.x as f32, p.y as f32))
            .collect();

        let mut indices = Vec::new();
        for (i, dir) in HEX_DIRECTIONS.iter().enumerate() {
            let neighbor = dir.add(tile_pos.0);

            if !tile_cache.tiles.contains_key(&TilePos(neighbor)) {
                indices.push([i as u32, (i as u32 + 1) % 6]);
            }
        }

        commands
            .entity(entity)
            .insert(Collider::polyline(
                corners,
                Some(indices),
                // Some(vec![[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 0]]),
            ))
            .insert(Transform::from_xyz(0.0, 0.0, 0.0))
            .insert(RigidBody::Fixed);

        // tiles_cache.tiles.insert(*tile_pos, entity);
    }
}

pub struct TilesPlugin;
impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileCache>()
            .add_system(spawn_tiles_system)
            .add_system(optimize_colliders_system);
    }
}
