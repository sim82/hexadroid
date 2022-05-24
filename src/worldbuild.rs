use crate::{
    hex_point_to_vec2,
    tiles::{TilePos, TileType, TilesState},
    HEX_LAYOUT,
};
use bevy::prelude::*;
use perlin2d::PerlinNoise2D;

fn init_walls_noise(mut commands: Commands, tiles_state: Res<TilesState>) {
    const SCALE: f64 = 256.0;
    let perlin = PerlinNoise2D::new(4, 1.0, 2.5, 1.0, 2.0, (SCALE, SCALE), 0.0, 101);
    const MAX: i32 = 10;
    const MIN: i32 = -MAX;

    for q in MIN..=MAX {
        for r in MIN..=MAX {
            let h = hexagon_tiles::hexagon::Hex::new(q, r);

            let p = hexagon_tiles::layout::LayoutTool::hex_to_pixel(HEX_LAYOUT, h);
            let p = hex_point_to_vec2(p);

            let noise = perlin.get_noise(p.x.into(), p.y.into());
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
pub struct WorldbuildPlugin;

impl Plugin for WorldbuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_walls_noise);
    }
}
