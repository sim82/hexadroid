use crate::{portal::PortalToggleRequest, prelude::*, HEX_LAYOUT};
use bevy::prelude::*;
use bevy_mouse_tracking_plugin::{MousePos, MousePosWorld};
use hexagon_tiles::{hexagon::HexRound, layout::LayoutTool, point::Point};
use std::io::Write;

fn mouse_input_system(
    mut commands: Commands,
    mouse_pos: Res<MousePosWorld>,
    buttons: Res<Input<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let hex_pos = LayoutTool::pixel_to_hex(
            HEX_LAYOUT,
            Point {
                x: mouse_pos.x.into(),
                y: mouse_pos.y.into(),
            },
        );
        // info!("press: {mouse_pos:?}")

        commands.spawn((
            TilePos(hex_pos.round()),
            // PortalToggleRequest::boundary_only(),
            PortalToggleRequest::default(),
        ));
    }
}

fn edit_command_system(input: Res<Input<KeyCode>>, tile_query: Query<&TilePos>) {
    if input.just_pressed(KeyCode::F5) {
        if let Ok(mut file) = std::fs::File::create("/tmp/level.txt") {
            for tile in &tile_query {
                write!(file, "{} {} {}\n", tile.0.q(), tile.0.r(), tile.0.s());
            }
        }
    }
}

pub struct EditPlugin;
impl Plugin for EditPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (mouse_input_system, edit_command_system));
    }
}
