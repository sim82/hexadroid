use bevy_rapier2d::prelude::Velocity;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use hexagon_tiles::{
    hexagon::{Hex, HEX_DIRECTIONS},
    layout::{Layout, LayoutTool, LAYOUT_ORIENTATION_POINTY},
    point::Point,
};

#[derive(Component, Default)]
pub struct InputTarget {
    pub direction: Vec2,
}

fn apply_input_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut InputTarget, &Transform)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for (mut input_target, transform) in query.iter_mut() {
        let direction = if keyboard_input.pressed(KeyCode::A) && keyboard_input.pressed(KeyCode::W)
        {
            HEX_DIRECTIONS[4]
        } else if keyboard_input.pressed(KeyCode::A) && keyboard_input.pressed(KeyCode::S) {
            HEX_DIRECTIONS[2]
        } else if keyboard_input.pressed(KeyCode::A) {
            HEX_DIRECTIONS[3]
        } else if keyboard_input.pressed(KeyCode::D) && keyboard_input.pressed(KeyCode::W) {
            HEX_DIRECTIONS[5]
        } else if keyboard_input.pressed(KeyCode::D) && keyboard_input.pressed(KeyCode::S) {
            HEX_DIRECTIONS[1]
        } else if keyboard_input.pressed(KeyCode::D) {
            HEX_DIRECTIONS[0]
        } else {
            Hex::new(0, 0)
        };
        let dir = LayoutTool::hex_to_pixel(
            Layout {
                orientation: LAYOUT_ORIENTATION_POINTY,
                size: Point { x: 64.0, y: 64.0 },
                origin: Point { x: 0.0, y: 0.0 },
            },
            direction,
        );
        input_target.direction = Vec2::new(dir.x as f32, dir.y as f32).normalize_or_zero();
        //* velocity = Velocity::linear(Vec2::new(dir.x as f32, dir.y as f32));
        // info!("dir: {:?}", dir);
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(apply_input_system);
    }
}
