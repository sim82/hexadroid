use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_lyon::{prelude::*, shapes};
use bevy_rapier2d::prelude::*;
use hexagon_tiles::{
    hexagon::HEX_DIRECTIONS,
    hexagon::{Hex, HexMath},
    layout::LayoutTool,
};

use crate::{debug::DebugLinesExt, hex_point_to_vec2, Despawn, HEX_LAYOUT};

#[derive(Component)]
pub struct TileType {
    pub wall: bool,
}

#[derive(Component, Eq, PartialEq, Copy, Clone, Debug)]
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
    pub dirty_set: HashSet<TilePos>,
}

fn spawn_tiles_system(
    mut commands: Commands,
    mut tiles_cache: ResMut<TileCache>,
    query: Query<(Entity, &TilePos, &TileType), Added<TileType>>,
    query_despawn: Query<(Entity, &TilePos), Added<Despawn>>,
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
        tiles_cache.dirty_set.insert(*tile_pos);
    }

    for (_entity, tile_pos) in query_despawn.iter() {
        info!("despawn: {:?}", tile_pos);
        tiles_cache.dirty_set.insert(*tile_pos);
        tiles_cache.tiles.remove(tile_pos);
    }
}

#[derive(Component)]
struct BoundaryMarker;

#[derive(Default)]
struct DedupEdges {
    points: Vec<Vec2>,
    edges: Vec<(usize, usize)>,
}

impl DedupEdges {}

fn optimize_colliders_system(
    mut commands: Commands,
    time: Res<Time>,
    mut delay: Local<f32>,
    mut tile_cache: ResMut<TileCache>,
    query: Query<(Entity, &TilePos, &TileType)>,
    mut debug_lines: ResMut<DebugLines>,
    boundary_query: Query<Entity, With<BoundaryMarker>>,
) {
    if *delay > 0.0 {
        *delay -= time.delta_seconds();
        return;
    }
    *delay = 0.5;
    if tile_cache.dirty_set.is_empty() {
        return;
    }

    info!("dirty: {:?}", tile_cache.dirty_set);

    let mut edges = Vec::new();

    for (entity, tile_pos, _) in query.iter() {
        let center = hex_point_to_vec2(LayoutTool::hex_to_pixel(HEX_LAYOUT, tile_pos.0));

        let corners: Vec<Vec2> = LayoutTool::polygon_corners(HEX_LAYOUT, Hex::new(0, 0))
            .iter()
            .map(|p| Vec2::new(p.x as f32, p.y as f32))
            .collect();

        let neighbors = [tile_pos.0; 6].zip(HEX_DIRECTIONS).map(|(a, b)| a.add(b));

        for (i, neighbor) in neighbors.iter().enumerate() {
            if !tile_cache.tiles.contains_key(&TilePos(*neighbor)) {
                let v1 = i;
                let v2 = (i + 1) % 6;
                edges.push((center + corners[v1], center + corners[v2]));
            }
        }

        if !(tile_cache.dirty_set.contains(tile_pos)
            || neighbors
                .iter()
                .any(|n| tile_cache.dirty_set.contains(&TilePos(*n))))
        {
            continue;
        }

        info!("dirty: {:?}", tile_pos);

        let mut indices = Vec::new();
        for (i, neighbor) in neighbors.iter().enumerate() {
            if !tile_cache.tiles.contains_key(&TilePos(*neighbor)) {
                let v1 = i;
                let v2 = (i + 1) % 6;
                indices.push([v1 as u32, v2 as u32]);
            }
        }

        commands
            .entity(entity)
            .insert(Collider::polyline(corners, Some(indices)))
            .insert(RigidBody::Fixed)
            .insert(Transform::from_translation(center.extend(0.0)));
    }
    // info!("edges: {:?}", edges);
    let mut num_pairs = 0;
    let shape = shapes::Circle {
        radius: 1.0,
        ..default()
    };

    let mut edge_pairs = HashMap::new();
    for (i, e) in edges.iter().enumerate() {
        for (j, f) in edges.iter().enumerate() {
            // if j <= i {
            //     continue;
            // }
            // const THRESHOLD: f32 = std::f32::EPSILON;
            const THRESHOLD: f32 = 1.0;
            if (e.0 - f.1).length() <= THRESHOLD {
                info!("pair {:?} {:?} -> {} {}", e, f, i, j);
                debug_lines.cross(e.0.extend(0.0), 10.0);
                edge_pairs.insert(i, j);
            }
        }
    }
    info!("num pairs: {}", edge_pairs.len());

    let mut edges_left = (0..edges.len()).collect::<HashSet<_>>();

    for entity in boundary_query.iter() {
        commands.entity(entity).despawn();
    }

    while !edges_left.is_empty() {
        let start_edge = *edges_left.iter().next().unwrap();

        let mut edge = start_edge;
        let mut x = 0;

        let mut points = Vec::new();
        loop {
            if x >= 100 {
                break;
            }
            x += 1;
            info!("edge: {}", edge);
            edges_left.remove(&edge);
            points.push(edges[edge].0);

            if let Some(next) = edge_pairs.get(&edge) {
                edge = *next;
            } else {
                info!("failed");
                break;
            }
            if edge == start_edge {
                break;
            }
        }

        let lyon_polygon = shapes::Polygon {
            closed: true,
            points: points.clone(),
        };

        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &lyon_polygon,
                DrawMode::Stroke(StrokeMode::new(Color::GREEN, 10.0)),
                default(),
            ))
            .insert(BoundaryMarker);
    }
    tile_cache.dirty_set.clear();
}

pub struct TilesPlugin;
impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileCache>()
            .add_system_to_stage(CoreStage::PostUpdate, spawn_tiles_system)
            .add_system(optimize_colliders_system);
    }
}
