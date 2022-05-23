use bevy::{
    prelude::*,
    utils::{HashMap, HashSet, Instant},
};
use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_lyon::{prelude::*, shapes};
use bevy_rapier2d::prelude::*;
use hexagon_tiles::{
    hexagon::HEX_DIRECTIONS,
    hexagon::{Hex, HexMath},
    layout::LayoutTool,
};

use crate::{debug::DebugLinesExt, hex_point_to_vec2, CmdlineArgs, Despawn, COLORS, HEX_LAYOUT};

pub struct TilesState {
    pub tile_root: Entity,
    edgeloop_root: Entity,
}

impl Default for TilesState {
    fn default() -> Self {
        Self {
            tile_root: Entity::from_raw(0), // meh, not really good but better than wrapping in Option...
            edgeloop_root: Entity::from_raw(0),
        }
    }
}

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
    pub dirty_set: HashSet<Entity>,
}

fn setup_system(
    mut commands: Commands,
    mut tiles_state: ResMut<TilesState>,
    args: Res<CmdlineArgs>,
) {
    tiles_state.tile_root = commands.spawn().insert(Name::new("tiles")).id();
    tiles_state.edgeloop_root = commands.spawn().insert(Name::new("edge_loops")).id();

    if !args.empty {
        for q in -5..=5 {
            for r in -5..=5 {
                let h = hexagon_tiles::hexagon::Hex::new(q, r);

                if q.abs() != 5 && r.abs() != 5 {
                    continue;
                }

                let entity = commands
                    .spawn()
                    .insert(TilePos(h))
                    .insert(TileType { wall: true })
                    .id();
                commands.entity(tiles_state.tile_root).add_child(entity);
            }
        }
    }
}

/// spawn initial collider for new tiles and update tile_cache (and dirty_set) for
/// spawned / despawend tiles
/// [`tile_cache.dirty_set`] is the set of directly affected tiles, i.e. the spawned
/// and despawned tiles and their neighbors.
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
    }

    for (entity, tile_pos) in query_despawn.iter() {
        info!("despawn: {:?}", tile_pos);
        dirty_add.push((entity, *tile_pos));
    }

    // add the modified tiles and their neighbors to dirty_set
    for (entity, pos) in dirty_add {
        tiles_cache.dirty_set.insert(entity);
        for n in [pos.0; 6].zip(HEX_DIRECTIONS).map(|(a, b)| a.add(b)) {
            let pos = TilePos(n);
            if let Some(ne) = tiles_cache.tiles.get(&pos).cloned() {
                tiles_cache.dirty_set.insert(ne);
            }
        }
    }
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

#[allow(clippy::too_many_arguments)]
fn optimize_colliders_system(
    mut commands: Commands,
    time: Res<Time>,
    mut delay: Local<f32>,
    mut tile_cache: ResMut<TileCache>,
    query: Query<&TilePos>,
    boundary_query: Query<(Entity, &BoundaryMarker)>,
    mut color_count: Local<usize>,
    mut debug_lines: Option<ResMut<DebugLines>>,
    tiles_state: Res<TilesState>,
) {
    if *delay > 0.0 {
        *delay -= time.delta_seconds();
        return;
    }
    *delay = 0.1;
    if tile_cache.dirty_set.is_empty() {
        return;
    }

    let start = Instant::now();

    // despawn outdated edge loops, i.e. those (also transitively) touched by dirty tiles
    let extended_dirty_set = despawn_dirty_edgeloops(
        std::mem::take(&mut tile_cache.dirty_set),
        boundary_query,
        &mut commands,
    );

    // spawn collider components (directly onto tile entities) and collect deduplicated edge set in [`dedup_edges`]
    let mut dedup_edges = util::DedupEdges::default();
    for entity in extended_dirty_set.iter() {
        // note: dirty set also contains entity ids of already despawned tiles, so we need to check this explicitly
        if let Ok(tile_pos) = query.get(*entity) {
            let center = hex_point_to_vec2(LayoutTool::hex_to_pixel(HEX_LAYOUT, tile_pos.0));

            if let Some(debug_lines) = debug_lines.as_mut() {
                debug_lines.cross(center.extend(0.0), 5.0);
            }

            let corners: Vec<Vec2> = LayoutTool::polygon_corners(HEX_LAYOUT, Hex::new(0, 0))
                .iter()
                .map(|p| Vec2::new(p.x as f32, p.y as f32))
                .collect();

            let neighbors = [tile_pos.0; 6].zip(HEX_DIRECTIONS).map(|(a, b)| a.add(b));

            let mut indices = Vec::new();

            // spawn colliders and collect edges only for solid -> free boundaries
            for (i, neighbor) in neighbors.iter().enumerate() {
                if !tile_cache.tiles.contains_key(&TilePos(*neighbor)) {
                    let v1 = i;
                    let v2 = (i + 1) % 6;
                    dedup_edges.add_edge(center + corners[v1], center + corners[v2], *entity);
                    indices.push([v1 as u32, v2 as u32]);
                }
            }

            trace!("dirty: {:?}", tile_pos);

            commands
                .entity(*entity)
                .insert(Collider::polyline(corners, Some(indices)))
                .insert(RigidBody::Fixed)
                .insert(Transform::from_translation(center.extend(0.0)));
        }
    }

    spawn_edgeloops(
        dedup_edges,
        *color_count,
        commands,
        tiles_state.edgeloop_root,
    );
    *color_count += 1;

    info!("update: {:?}", start.elapsed());
}

/// despawn bundary loops that are affected by [`dirty_set`].
///
/// also create extended dirty_set which includes all tiles from the despawned loops
///  => we need to to generate the new bundary loops.
/// this algorithm uses multiple passes since the dirty set from one loop can influence new loops
/// that were not touched by the initial dirty set etc.
fn despawn_dirty_edgeloops(
    mut dirty_set: HashSet<Entity>,
    boundary_query: Query<(Entity, &BoundaryMarker)>,
    commands: &mut Commands,
) -> bevy::utils::hashbrown::HashSet<Entity> {
    info!("dirty: {:?}", dirty_set);
    let mut removed_boundaries = HashSet::new();
    let mut loops = 0;
    loop {
        let mut extend: HashSet<Entity> = default();
        for (entity, boundary) in boundary_query.iter() {
            trace!("test: {:?} {:?}", boundary.tiles, removed_boundaries);

            if !removed_boundaries.contains(&entity) && !boundary.tiles.is_disjoint(&dirty_set) {
                commands.entity(entity).despawn();
                trace!("despawn: {:?}", boundary.tiles);
                // extended_dirty_set.extend(boundary.tiles.iter());
                extend.extend(boundary.tiles.iter());
                removed_boundaries.insert(entity);
            }
        }
        let stop = extend.is_empty();
        dirty_set.extend(extend.drain());
        if stop {
            break;
        }
        loops += 1;
    }
    info!("new dirty: {:?} {}", dirty_set, loops);
    dirty_set
}

fn spawn_edgeloops(
    dedup_edges: util::DedupEdges,
    color_count: usize,
    mut commands: Commands,
    root: Entity,
) {
    let mut edge_pairs = HashMap::new();
    for (i, (e0, _, _)) in dedup_edges.edges.iter().enumerate() {
        for (j, (_, f1, _)) in dedup_edges.edges.iter().enumerate() {
            if e0 == f1 {
                trace!("pair: {} {}", i, j);
                edge_pairs.insert(i, j);
            }
        }
    }
    info!("num pairs: {}", edge_pairs.len());
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
            let edge_p0 = dedup_edges.points[p0];
            let offs = if false {
                Vec2::X * (color_count as f32)
            } else {
                default()
            };
            points.push(edge_p0 + offs);
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

        let entity = commands
            .spawn_bundle(GeometryBuilder::build_as(
                &lyon_polygon,
                DrawMode::Stroke(StrokeMode::new(COLORS[color_count % COLORS.len()], 10.0)),
                default(),
            ))
            .insert(BoundaryMarker { tiles })
            .id();

        commands.entity(root).add_child(entity);
    }
}

pub struct TilesPlugin;
impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileCache>()
            .init_resource::<TilesState>()
            .add_startup_system(setup_system)
            .add_system_to_stage(CoreStage::PostUpdate, spawn_tiles_system)
            .add_system(optimize_colliders_system);
    }
}
