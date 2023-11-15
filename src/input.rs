use crate::{
    droid::{AttackRequest, TargetDirection},
    hexton::HextonInput,
    portal::PortalToggleRequest,
    prelude::*,
    ship::ShipInput, HEX_LAYOUT,
};

use bevy::{math::Vec3Swizzles, prelude::*};
// use bevy_mouse_tracking_plugin::prelude::*;
// use bevy_mouse_tracking_plugin::MousePosWorld;
use bevy_rapier2d::prelude::*;
use hexagon_tiles::{
    hexagon::{HexMath, HexRound},
    layout::LayoutTool,
    point::Point,
};


#[derive(Component, Default)]
pub struct InputTarget;

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
fn apply_input_system_portal_toggle(
    mut commands: Commands,
    query: Query<&Transform, With<InputTarget>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for transform in &query {
        if keyboard_input.just_pressed(KeyCode::H) {
            let hex_pos = LayoutTool::pixel_to_hex(
                HEX_LAYOUT,
                Point {
                    x: transform.translation.x.into(),
                    y: transform.translation.y.into(),
                },
            );
            commands.spawn((TilePos(hex_pos.round()), PortalToggleRequest));
        }
    }
}
fn apply_input_system_2_1_dof(
    mut query: Query<(&mut ShipInput, &Transform, &mut AttackRequest), With<InputTarget>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for (mut input_target, _transform, mut attack_request) in query.iter_mut() {
        let w = keyboard_input.pressed(KeyCode::W);
        let a = keyboard_input.pressed(KeyCode::A);
        let s = keyboard_input.pressed(KeyCode::S);
        let d = keyboard_input.pressed(KeyCode::D);

        let mut rot = 0.0;
        let mut thrust = 0.0;
        let mut brake = 0.0;
        if w {
            thrust += 1.0;
        }
        if s {
            brake += 1.0;
        }
        if d {
            rot += 1.0;
        }
        if a {
            rot -= 1.0;
        }

        input_target.rot = rot;
        input_target.thrust = thrust;
        input_target.brake = brake;

        attack_request.primary_attack = keyboard_input.pressed(KeyCode::J);
    }
}

fn apply_input_system_jnr(
    mut query: Query<(&mut HextonInput, &mut AttackRequest), With<InputTarget>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for (mut input_target, mut attack_request) in query.iter_mut() {
        let mut forward = 0.0;
        let w = keyboard_input.pressed(KeyCode::W);
        let a = keyboard_input.pressed(KeyCode::A);
        // let s = keyboard_input.pressed(KeyCode::S);
        let d = keyboard_input.pressed(KeyCode::D);

        if d {
            forward += 1.0;
        }
        if a {
            forward -= 1.0;
        }
        input_target.forward = forward;
        input_target.jump = w;
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

pub enum ClickMode {
    TileAddRemove,
    WaypointSample,
}
// FIXME: disabled due to missing mouse track plugin
fn background_on_click_system() {}
// fn background_on_click_system(
//     mut commands: Commands,
//     mouse: Res<MousePosWorld>,
//     // mut pointer_pos: Local<Vec2>,
//     mut mouse_button_input_events: EventReader<MouseButtonInput>,
//     mut tile_cache: ResMut<TileCache>,
//     tiles_state: Res<TilesState>,
//     mut waypoints_gui_state: ResMut<waypoint::GuiState>, // FIXME: make click_handler mode specific.
// ) {
//     for button_event in mouse_button_input_events.iter() {
//         if button_event.button == MouseButton::Left && button_event.state == ButtonState::Released {
//             // info!("click: {:?}", pointer_pos);
//             let hex = LayoutTool::pixel_to_hex(
//                 HEX_LAYOUT,
//                 Point {
//                     x: mouse.x as f64,
//                     y: mouse.y as f64,
//                 },
//             )
//             .round();
//             info!("mouse pos: {:?} -> {:?}", *mouse, hex);

//             let tile_pos = TilePos(hex);
//             let click_mode = ClickMode::TileAddRemove;

//             match click_mode {
//                 ClickMode::TileAddRemove => {
//                     if let Some(entity) = tile_cache.tiles.remove(&tile_pos) {
//                         info!("delete");
//                         commands.entity(entity).insert(Despawn::ThisFrame);
//                     } else {
//                         info!("spawn");
//                         let entity = commands
//                             .spawn(SpatialBundle::default())
//                             .insert(TileType {
//                                 wall: true,
//                                 immediate_collider: true,
//                             })
//                             .insert(TilePos(hex))
//                             .id();
//                         commands.entity(tiles_state.tile_root).add_child(entity);
//                     }
//                 }
//                 ClickMode::WaypointSample => {
//                     // let pattern = [tile_pos.0; 6]
//                     //     .zip(HEX_DIRECTIONS)
//                     //     .map(|(a, b)| TilePos(a.add(b)))
//                     //     .map(|p| tile_cache.tiles.contains_key(&p));

//                     let mut pattern = [false; 6];
//                     for i in 0..6 {
//                         pattern[i] = tile_cache
//                             .tiles
//                             .contains_key(&TilePos(tile_pos.0.add(HEX_DIRECTIONS[i])));
//                     }
//                     // add or remove (NOTE: is there a better pattern for this?)
//                     let is_new = waypoints_gui_state.rules2.insert(pattern);
//                     if !is_new {
//                         waypoints_gui_state.rules2.remove(&pattern);
//                     }

//                     let mut pattern_sorted = waypoints_gui_state
//                         .rules2
//                         .iter()
//                         .cloned()
//                         .collect::<Vec<_>>();
//                     pattern_sorted.sort();
//                     let mut f = std::fs::File::create("pattern.txt").unwrap();
//                     for p in pattern_sorted {
//                         let p = p.map(i32::from);
//                         let _ = writeln!(f, "{}{}{}{}{}{}", p[0], p[1], p[2], p[3], p[4], p[5]);
//                     }

//                     waypoints_gui_state.update = true;
//                 }
//             }
//         }
//     }
// }

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                apply_input_system_8dir,
                apply_input_system_2_1_dof,
                apply_input_system_jnr,
                background_on_click_system,
                camera_zoom_system,
                camera_rotate_system,
                apply_input_system_portal_toggle,
            ),
        )
        // .add_plugins(MousePosPlugin)
        ;
    }
}
