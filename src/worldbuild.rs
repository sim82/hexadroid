use crate::{
    camera::CameraTarget,
    hex_point_to_vec2,
    tiles::{TileCache, TilePos, TileType, TilesState},
    vec2_to_hex_point, Despawn, HEX_LAYOUT,
};
use bevy::{math::Vec3Swizzles, prelude::*};
use hexagon_tiles::hexagon::{Hex, HexMath, HexRound};
use perlin2d::PerlinNoise2D;

pub struct WorldState {
    min: Hex,
    max: Hex,

    pub min_target: Hex,
    pub max_target: Hex,

    center: Hex,

    perlin: PerlinNoise2D,
}

const PERLIN_SCALE: f64 = 256.0;
const INITIAL_SIZE: i32 = 2;

impl Default for WorldState {
    fn default() -> Self {
        Self {
            min: Hex::new(0, 0),
            max: Hex::new(0, 0),
            min_target: Hex::new(-INITIAL_SIZE, -INITIAL_SIZE),
            max_target: Hex::new(INITIAL_SIZE, INITIAL_SIZE),
            center: Hex::new(0, 0),
            perlin: PerlinNoise2D::new(
                4,
                1.0,
                1.5,
                2.0,
                2.0,
                (PERLIN_SCALE, PERLIN_SCALE),
                -0.4,
                101,
            ),
        }
    }
}

fn update_walls_noise(
    mut commands: Commands,
    tiles_state: Res<TilesState>,
    tiles_cache: Res<TileCache>,
    mut world_state: ResMut<WorldState>,
    camera_query: Query<&Transform, With<CameraTarget>>,
) {
    if let Ok(Transform { translation, .. }) = camera_query.get_single() {
        let point = vec2_to_hex_point(translation.xy());
        let hex = hexagon_tiles::layout::LayoutTool::pixel_to_hex(HEX_LAYOUT, point).round();
        // info!("center: {:?}", hex);

        let dist = world_state.center.sub(hex);
        if dist.q().abs() >= 2 || dist.r().abs() >= 2 {
            info!("move center {:?}", hex);
            world_state.max_target = world_state.max_target.sub(dist);
            world_state.min_target = world_state.min_target.sub(dist);
            world_state.center = hex;
        }
    }

    if world_state.max_target == world_state.max && world_state.min_target == world_state.min {
        return;
    }

    let valid_range_q = world_state.min_target.q()..=world_state.max_target.q();
    let valid_range_r = world_state.min_target.r()..=world_state.max_target.r();

    let old_range_q = world_state.min.q()..=world_state.max.q();
    let old_range_r = world_state.min.r()..=world_state.max.r();

    // if world_state.max_target < world_state.max {
    // shrink
    for (pos, entity) in tiles_cache.tiles.iter() {
        let q = pos.0.q();
        let r = pos.0.r();

        if !valid_range_q.contains(&q) || !valid_range_r.contains(&r) {
            commands.entity(*entity).insert(Despawn::ThisFrame);
        }
    }
    // } else {
    // grow
    for q in valid_range_q {
        for r in valid_range_r.clone() {
            // if r >= world_state.min
            //     && r <= world_state.max
            //     && q >= world_state.min
            //     && q <= world_state.max
            if old_range_q.contains(&q) && old_range_r.contains(&r) {
                continue;
            }
            let h = hexagon_tiles::hexagon::Hex::new(q, r);

            let p = hexagon_tiles::layout::LayoutTool::hex_to_pixel(HEX_LAYOUT, h);
            let p = hex_point_to_vec2(p);

            let noise = world_state.perlin.get_noise(p.x.into(), p.y.into());
            // info!("noise {} {} {} {}", q, r, p, noise);

            if noise < 0.0 {
                continue;
            }

            let h = hexagon_tiles::hexagon::Hex::new(q, r);
            // if q.abs() != 5 && r.abs() != 5 {
            //     continue;
            // }

            let entity = commands
                .spawn()
                .insert(TilePos(h))
                .insert(TileType { wall: true })
                .id();
            commands.entity(tiles_state.tile_root).add_child(entity);
        }
    }
    // }
    world_state.min = world_state.min_target;
    world_state.max = world_state.max_target;
}
pub struct WorldbuildPlugin;

impl Plugin for WorldbuildPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldState>()
            .add_system(update_walls_noise);
    }
}
