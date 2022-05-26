use std::io::BufWriter;

use bevy_rapier2d::prelude::Velocity;

use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    math::Vec3Swizzles,
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
    tiles::{TileCache, TilePos, TileType, TilesState},
    worldbuild::WorldState,
    Despawn, HEX_LAYOUT,
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

fn apply_input_system_8dir(
    mut query: Query<(&mut TargetDirection, &Transform, &mut AttackRequest), With<InputTarget>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for (mut input_target, _transform, mut attack_request) in query.iter_mut() {
        let w = keyboard_input.pressed(KeyCode::W);
        let a = keyboard_input.pressed(KeyCode::A);
        let s = keyboard_input.pressed(KeyCode::S);
        let d = keyboard_input.pressed(KeyCode::D);

        let mut dir = Vec2::ZERO;
        if w {
            dir += Vec2::Y;
        }
        if s {
            dir -= Vec2::Y;
        }
        if d {
            dir += Vec2::X;
        }
        if a {
            dir -= Vec2::X;
        }

        input_target.direction = dir.normalize_or_zero();
        attack_request.primary_attack = keyboard_input.pressed(KeyCode::J);
    }
}

fn camera_zoom_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    for mut transform in camera_query.iter_mut() {
        if keyboard_input.pressed(KeyCode::I) && transform.scale.x > 0.5 {
            transform.scale /= 1.05
        } else if keyboard_input.pressed(KeyCode::P) && transform.scale.x < 10.0 {
            transform.scale *= 1.05
        } else if keyboard_input.pressed(KeyCode::O) {
            transform.scale = Vec3::ONE
        }
    }
}

fn camera_rotate_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // info!("scale: {:?}", rapier_config);

    for mut transform in camera_query.iter_mut() {
        let angle = 3.0 * std::f32::consts::PI / 360.0;

        if keyboard_input.pressed(KeyCode::K) {
            let rot = Quat::from_rotation_z(angle);
            transform.rotation *= rot;
            rapier_config.gravity = rot.mul_vec3(rapier_config.gravity.extend(0.0)).xy();
        } else if keyboard_input.pressed(KeyCode::L) {
            let rot = Quat::from_rotation_z(-angle);
            transform.rotation *= rot;
            rapier_config.gravity = rot.mul_vec3(rapier_config.gravity.extend(0.0)).xy();
        }
    }
}

fn world_debug_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    mut world_state: ResMut<WorldState>,
) {
    // info!("scale: {:?}", rapier_config);

    const INCREMENT: i32 = 5;
    if keyboard_input.just_pressed(KeyCode::Y) {
        if world_state.max_target > 1 {
            world_state.max_target -= INCREMENT;
            world_state.min_target += INCREMENT;
        }
    } else if keyboard_input.just_pressed(KeyCode::U) {
        if world_state.max_target < 40 {
            world_state.max_target += INCREMENT;
            world_state.min_target -= INCREMENT;
        }
    } else if keyboard_input.just_pressed(KeyCode::Q) {
        if let Ok(file) = std::fs::File::create("physics.yaml") {
            let _ = serde_yaml::to_writer(BufWriter::new(file), &*rapier_context);
        }
    }
}

fn background_on_click_system(
    mut commands: Commands,
    mouse: Res<MousePosWorld>,
    // mut pointer_pos: Local<Vec2>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut tile_cache: ResMut<TileCache>,
    tiles_state: Res<TilesState>,
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

            let tile_pos = TilePos(hex);
            if let Some(entity) = tile_cache.tiles.remove(&tile_pos) {
                info!("delete");
                commands.entity(entity).insert(Despawn::ThisFrame);
            } else {
                info!("spawn");
                let entity = commands
                    .spawn()
                    .insert(TileType { wall: true })
                    .insert(TilePos(hex))
                    .id();
                commands.entity(tiles_state.tile_root).add_child(entity);
            }
        }
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(apply_input_system_8dir)
            .add_system(background_on_click_system)
            .add_system(camera_zoom_system)
            .add_system(camera_rotate_system)
            .add_system(world_debug_input_system)
            .add_plugin(MousePosPlugin::SingleCamera);
    }
}
