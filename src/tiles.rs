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

use crate::{debug::DebugLinesExt, hex_point_to_vec2, Despawn, COLORS, HEX_LAYOUT};

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
    pub dirty_entity_set: HashSet<Entity>,
}

fn spawn_tiles_system(
    mut commands: Commands,
    mut tiles_cache: ResMut<TileCache>,
    query: Query<(Entity, &TilePos, &TileType), Added<TileType>>,
    query_despawn: Query<(Entity, &TilePos), Added<Despawn>>,
) {
    let mut dirty_add = Vec::new();
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
        dirty_add.push((entity, *tile_pos));
        // tiles_cache.dirty_set.insert(*tile_pos);
        // tiles_cache.dirty_entity_set.insert(entity);
    }

    for (entity, tile_pos) in query_despawn.iter() {
        info!("despawn: {:?}", tile_pos);
        tiles_cache.dirty_set.insert(*tile_pos);
        // tiles_cache.tiles.remove(tile_pos);
        // tiles_cache.dirty_entity_set.insert(entity);
        dirty_add.push((entity, *tile_pos));
    }

    for (add_entity, add_pos) in dirty_add {
        tiles_cache.dirty_set.insert(add_pos);
        tiles_cache.dirty_entity_set.insert(add_entity);

        for n in [add_pos.0; 6].zip(HEX_DIRECTIONS).map(|(a, b)| a.add(b)) {
            let pos = TilePos(n);
            if let Some(ne) = tiles_cache.tiles.get(&pos).cloned() {
                tiles_cache.dirty_set.insert(pos);
                tiles_cache.dirty_entity_set.insert(ne);
            }
        }
    }
    // let mut dirty_neighbors = HashSet::new();
    // let mut dirty_neighbor_entities = HashSet::new();
    // for dirty in tiles_cache.dirty_set.iter() {}
    // tiles_cache.dirty_entity_set.extend(dirty_neighbor_entities);
    // tiles_cache.dirty_set.extend(dirty_neighbors);

    // info!("spawn tiles dirty: {:?}", tiles_cache.dirty_entity_set);
}

#[derive(Component, Default)]
struct BoundaryMarker {
    tiles: HashSet<Entity>,
}

pub mod util {
    use bevy::prelude::*;
    #[derive(Default)]
    pub struct DedupEdges {
        pub points: Vec<Vec2>,
        pub edges: Vec<(usize, usize, Entity)>,
    }

    impl DedupEdges {
        const THRESHOLD: f32 = 1.0;

        pub fn get_or_insert_point(&mut self, p: Vec2) -> usize {
            for (i, r) in self.points.iter().enumerate() {
                if (p - *r).length() <= Self::THRESHOLD {
                    return i;
                }
            }
            self.points.push(p);
            self.points.len() - 1
        }
        pub fn get_point_by_index(&self, i: usize) -> Vec2 {
            self.points[i]
        }
        pub fn add_edge(&mut self, a: Vec2, b: Vec2, owner: Entity) {
            let e = (
                self.get_or_insert_point(a),
                self.get_or_insert_point(b),
                owner,
            );
            self.edges.push(e);
        }
        pub fn get_edge_p0(&self, edge: usize) -> Vec2 {
            let (p0, _, _) = self.edges[edge];
            self.points[p0]
        }
    }
}
fn optimize_colliders_system(
    mut commands: Commands,
    time: Res<Time>,
    mut delay: Local<f32>,
    mut tile_cache: ResMut<TileCache>,
    query: Query<(Entity, &TilePos, &TileType)>,
    boundary_query: Query<(Entity, &BoundaryMarker)>,
    mut color_count: Local<usize>,
) {
    if *delay > 0.0 {
        *delay -= time.delta_seconds();
        return;
    }
    *delay = 0.5;
    if tile_cache.dirty_set.is_empty() {
        return;
    }

    // info!("dirty: {:?}", tile_cache.dirty_set);
    let mut dedup_edges = util::DedupEdges::default();

    // for entity in tile_cache.dirty_entity_set.iter() {
    // let (_, tile_pos, _) = query.get(*entity).unwrap();
    // }

    info!("dirty: {:?}", tile_cache.dirty_entity_set);
    let mut dirty_entities: HashSet<Entity> = default();
    for (entity, boundary) in boundary_query.iter() {
        info!("test: {:?}", boundary.tiles);

        if !boundary.tiles.is_disjoint(&tile_cache.dirty_entity_set) {
            commands.entity(entity).despawn();
            info!("despawn: {:?}", boundary.tiles);
            dirty_entities.extend(boundary.tiles.iter());
        }
    }

    dirty_entities.extend(tile_cache.dirty_entity_set.iter());

    info!("new dirty: {:?}", dirty_entities);

    for (ref entity, tile_pos, _) in query.iter() {
        let center = hex_point_to_vec2(LayoutTool::hex_to_pixel(HEX_LAYOUT, tile_pos.0));

        let corners: Vec<Vec2> = LayoutTool::polygon_corners(HEX_LAYOUT, Hex::new(0, 0))
            .iter()
            .map(|p| Vec2::new(p.x as f32, p.y as f32))
            .collect();

        let neighbors = [tile_pos.0; 6].zip(HEX_DIRECTIONS).map(|(a, b)| a.add(b));

        if !(
            dirty_entities.contains(entity)
            // || neighbors
            //     .iter()
            //     .any(|n| tile_cache.dirty_set.contains(&TilePos(*n)))
        ) {
            continue;
        }

        let mut indices = Vec::new();

        for (i, neighbor) in neighbors.iter().enumerate() {
            if !tile_cache.tiles.contains_key(&TilePos(*neighbor)) {
                let v1 = i;
                let v2 = (i + 1) % 6;
                dedup_edges.add_edge(center + corners[v1], center + corners[v2], *entity);
                indices.push([v1 as u32, v2 as u32]);
            }
        }

        info!("dirty: {:?}", tile_pos);

        commands
            .entity(*entity)
            .insert(Collider::polyline(corners, Some(indices)))
            .insert(RigidBody::Fixed)
            .insert(Transform::from_translation(center.extend(0.0)));
    }

    let mut edge_pairs = HashMap::new();
    for (i, (e0, _, _)) in dedup_edges.edges.iter().enumerate() {
        for (j, (_, f1, _)) in dedup_edges.edges.iter().enumerate() {
            if e0 == f1 {
                trace!("pair: {} {}", i, j);
                edge_pairs.insert(i, j);
            }
        }
    }
    // info!("num pairs: {}", edge_pairs.len());

    // trace all connected edge loops to generate polygons for rendering
    let mut edges_left = (0..dedup_edges.edges.len()).collect::<HashSet<_>>();

    while !edges_left.is_empty() {
        let mut tiles = HashSet::new();
        let start_edge = *edges_left.iter().next().unwrap();
        let mut edge = start_edge;
        let mut points = Vec::new();
        loop {
            trace!("edge: {}", edge);
            let was_removed = edges_left.remove(&edge);
            if !was_removed {
                // this can only mean that an edge was reached twice while tracing the loop.
                // should not be possible in our case since edges of a loop cannot cross etc.
                error!("edge not in edges_left set.");
                break;
            }
            // points.push(edges[edge].0);
            let (p0, _, tile_entity) = dedup_edges.edges[edge];
            points.push(dedup_edges.points[p0] + Vec2::X * (*color_count as f32));
            tiles.insert(tile_entity);
            // points.push(dedup_edges.get_edge_p0(edge));
            if let Some(next) = edge_pairs.get(&edge) {
                edge = *next;
            } else {
                // no neighboring edge found. should not be possible if all edges are loops.
                error!("loop not closed");
                break;
            }
            if edge == start_edge {
                // reached start of loop. all is well.
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
                DrawMode::Stroke(StrokeMode::new(COLORS[*color_count % COLORS.len()], 10.0)),
                default(),
            ))
            .insert(BoundaryMarker { tiles });
    }
    *color_count += 1;
    tile_cache.dirty_set.clear();
    tile_cache.dirty_entity_set.clear();
}

pub struct TilesPlugin;
impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileCache>()
            .add_system_to_stage(CoreStage::PostUpdate, spawn_tiles_system)
            .add_system(optimize_colliders_system);
    }
}
