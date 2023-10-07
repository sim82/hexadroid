use bevy::{prelude::*, utils::HashSet};
use bevy_egui::{egui, EguiContext};
use bevy_prototype_debug_lines::DebugLines;
use hexagon_tiles::{
    hexagon::{HexMath, HEX_DIRECTIONS},
    layout::LayoutTool,
};

use crate::{
    debug::DebugLinesExt,
    hex_point_to_vec2,
    tiles::{TileCache, TilePos},
    HEX_LAYOUT,
};

#[derive(Default, Resource)]
pub struct GuiState {
    rules: [bool; 6],
    pub rules2: HashSet<[bool; 6]>,
    pub update: bool,
}

#[derive(Component)]
pub struct Waypoint;

fn waypoint_egui_system(mut state: ResMut<GuiState>, mut egui_context: Query<&EguiContext>) {
    let Ok(egui_context) = egui_context.get_single() else {
        return;
    };
    egui::Window::new("waypoint").show(egui_context.get_mut(), |ui| {
        for (i, r) in state.rules.iter_mut().enumerate() {
            ui.checkbox(r, format!("rule {i}"));
        }

        state.update = ui.button("update").clicked();
    });
}

fn waypoint_update_system(
    tiles_cache: Res<TileCache>,
    mut state: ResMut<GuiState>,
    mut debug_lines: Option<ResMut<DebugLines>>,
) {
    if !state.update {
        return;
    }
    state.update = false;

    // let neighbors = [tile_pos.0; 6].zip(HEX_DIRECTIONS).map(|(a, b)| a.add(b));
    // let candidates = tiles_cache
    //     .tiles
    //     .iter()
    //     .flat_map(|(tile_pos, _)| {
    //         [tile_pos.0; 6]
    //             .iter()
    //             .enumerate()
    //             .zip(HEX_DIRECTIONS)
    //             .map(|((i, a), b)| (i, TilePos(a.add(b))))
    //             .collect::<Vec<_>>()
    //     })
    //     .filter_map(|(i, p)| {
    //         if state.rules[i] && !tiles_cache.tiles.contains_key(&p) {
    //             Some(p)
    //         } else {
    //             None
    //         }
    //     })
    //     .collect::<HashSet<_>>();
    let candidates = tiles_cache
        .tiles
        .keys()
        .flat_map(|tile_pos| {
            // [tile_pos.0; 6]
            //     .zip(HEX_DIRECTIONS)
            //     .map(|(a, b)| TilePos(a.add(b)))
            // let mut ret = [tile_pos; 6];
            // for i in 0..6 {
            //     ret[i] = TilePos(tile_pos.0.add(HEX_DIRECTIONS[i]));
            // }
            // ret.into_iter()
            tile_pos.get_neighbors()
        })
        .collect::<HashSet<_>>();

    // let candidates = tiles_cache
    //     .tiles
    //     .keys()
    //     .flat_map(|tile_pos| {
    //         [tile_pos.0; 6]
    //             .zip(HEX_DIRECTIONS)
    //             .map(|(a, b)| TilePos(a.add(b)))
    //     })
    //     .filter(|tile_pos| {
    //         if tiles_cache.tiles.contains_key(tile_pos) {
    //             return false;
    //         }
    //         let num = [tile_pos.0; 6]
    //             .zip(HEX_DIRECTIONS)
    //             .iter()
    //             .filter(|(a, b)| !tiles_cache.tiles.contains_key(&TilePos(a.add(*b))))
    //             .count();
    //         state.rules[num]
    //     })
    //     // .flat_map(|(tile_pos, _)| {
    //     //     [tile_pos.0; 6]
    //     //         .iter()
    //     //         .enumerate()
    //     //         .zip(HEX_DIRECTIONS)
    //     //         .map(|((i, a), b)| (i, TilePos(a.add(b))))
    //     //         .collect::<Vec<_>>()
    //     // })
    //     // .filter_map(|(i, p)| {
    //     //     if state.rules[i] && !tiles_cache.tiles.contains_key(&p) {
    //     //         Some(p)
    //     //     } else {
    //     //         None
    //     //     }
    //     // })
    //     .collect::<HashSet<_>>();

    info!("candidates: {}", candidates.len());
    for p in candidates.iter() {
        // let pattern = [p.0; 6]
        //     .zip(HEX_DIRECTIONS)
        //     .map(|(a, b)| TilePos(a.add(b)))
        //     .map(|p| tiles_cache.tiles.contains_key(&p));
        let mut pattern = [false; 6];
        let neighbors = p.get_neighbors();
        for i in 0..6 {
            pattern[i] = tiles_cache.tiles.contains_key(&neighbors[i]);
        }

        if tiles_cache.tiles.contains_key(p) || !state.rules2.contains(&pattern) {
            continue;
        }

        let center = hex_point_to_vec2(LayoutTool::hex_to_pixel(HEX_LAYOUT, p.0)).extend(0.0);
        if let Some(debug_lines) = debug_lines.as_mut() {
            debug_lines.cross(center, 15.0);
        }
    }
}

pub struct WaypointPlugin;
impl Plugin for WaypointPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuiState>()
            .add_system(waypoint_egui_system)
            .add_system(waypoint_update_system);
    }
}
