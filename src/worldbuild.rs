use crate::{
    hex_point_to_vec2,
    tiles::{TileCache, TilePos, TileType, TilesState},
    Despawn, HEX_LAYOUT,
};
use bevy::prelude::*;
use perlin2d::PerlinNoise2D;

pub struct WorldState {
    min: i32,
    max: i32,

    pub min_target: i32,
    pub max_target: i32,

    perlin: PerlinNoise2D,
}

const PERLIN_SCALE: f64 = 256.0;
const INITIAL_SIZE: i32 = 20;

impl Default for WorldState {
    fn default() -> Self {
        Self {
            min: 0,
            max: 0,
            min_target: -INITIAL_SIZE,
            max_target: INITIAL_SIZE,
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
) {
    if world_state.max_target == world_state.max && world_state.min_target == world_state.min {
        return;
    }
    let valid_range = world_state.min_target..=world_state.max_target;

    if world_state.max_target < world_state.max {
        // shrink
        for (pos, entity) in tiles_cache.tiles.iter() {
            let q = pos.0.q();
            let r = pos.0.r();

            if !valid_range.contains(&q) || !valid_range.contains(&r) {
                commands.entity(*entity).insert(Despawn::ThisFrame);
            }
        }
    } else {
        // grow
        for q in world_state.min_target..=world_state.max_target {
            for r in world_state.min_target..=world_state.max_target {
                if r >= world_state.min
                    && r <= world_state.max
                    && q >= world_state.min
                    && q <= world_state.max
                {
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
    }
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
