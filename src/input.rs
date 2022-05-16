use bevy_rapier2d::prelude::Velocity;

use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    prelude::*,
    render::camera::Camera2d,
};
use bevy_mouse_tracking_plugin::{MousePos, MousePosPlugin, MousePosWorld};
use bevy_rapier2d::prelude::*;
use hexagon_tiles::{
    hexagon::{Hex, HexRound, HEX_DIRECTIONS},
    layout::{Layout, LayoutTool, LAYOUT_ORIENTATION_POINTY},
    point::Point,
};

use crate::{
    droid::{AttackRequest, TargetDirection},
    hex_point_to_vec2,
    tiles::{TilePos, TileType},
    HEX_LAYOUT,
};

#[derive(Component, Default)]
pub struct InputTarget;

fn apply_input_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut TargetDirection, &Transform, &mut AttackRequest), With<InputTarget>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for (mut input_target, transform, mut attack_request) in query.iter_mut() {
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
        input_target.direction =
            hex_point_to_vec2(LayoutTool::hex_to_pixel(HEX_LAYOUT, direction)).normalize_or_zero();
        //* velocity = Velocity::linear(Vec2::new(dir.x as f32, dir.y as f32));
        // info!("dir: {:?}", input_target.direction);

        attack_request.primary_attack = keyboard_input.pressed(KeyCode::J);
    }
}

fn background_on_click_system(
    mut commands: Commands,
    mouse: Res<MousePosWorld>,
    // mut pointer_pos: Local<Vec2>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    // mut cursor_moved_events: EventReader<CursorMoved>,
    // cam_2d_query: Query<(&GlobalTransform, &Camera), With<Camera2d>>
) {
    for button_event in mouse_button_input_events.iter() {
        if button_event.button == MouseButton::Left && button_event.state == ElementState::Released
        {
            // info!("click: {:?}", pointer_pos);
            let hex = LayoutTool::pixel_to_hex(
                HEX_LAYOUT,
                Point {
                    x: mouse.x as f64,
                    y: mouse.y as f64,
                },
            )
            .round();
            info!("mouse pos: {:?} -> {:?}", *mouse, hex);
            commands
                .spawn()
                .insert(TileType { wall: true })
                .insert(TilePos(hex));
        }
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(apply_input_system)
            .add_system(background_on_click_system)
            .add_plugin(MousePosPlugin::SingleCamera);
    }
}
