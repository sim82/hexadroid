// commander Chuck Hexton

use std::{borrow::Cow, time::Duration};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{collision_groups, droid::AttackRequest};

#[derive(Component, Default)]
pub struct HextonInput {
    pub forward: f32,
    pub jump: bool,
}

#[derive(Component)]
pub struct HextonGravity {
    pub speed: f32,
    pub jump: bool,
    pub jump_timer: Timer,
    // pub jump: bool,
}

impl Default for HextonGravity {
    fn default() -> Self {
        Self {
            speed: Default::default(),
            jump: Default::default(),
            jump_timer: Timer::new(Duration::from_millis(300), false),
        }
    }
}

#[derive(Bundle)]
pub struct HextonBundle {
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    // pub transform: Transform,
    pub external_force: ExternalForce,
    // pub external_impulse: ExternalImpulse,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    // pub friction: Friction,
    // pub restitution: Restitution,
    pub gravity: HextonGravity,
    pub name: Name,
    // pub ground_friction: GroundFriction,
    // pub weapon_direction: WeaponDirection,
    // pub weapon_state: WeaponState,
    pub hexton_input: HextonInput,
    // pub ship_thruster: ShipThruster,
    pub attack_request: AttackRequest,
    // pub damping: Damping,
    // pub read_mass_properties: ReadMassProperties,
    pub mass_properties: ColliderMassProperties,
    pub character_controller: KinematicCharacterController,
    pub controller_output: KinematicCharacterControllerOutput,

    #[bundle]
    pub spatial_bundle: SpatialBundle,
}

pub const HEXTON_VERTICES: [Vec2; 3] = [
    Vec2::new(0.0, 10.0),
    Vec2::new(-5.0, -10.0),
    Vec2::new(5.0, -10.0),
];

pub const SHIP_MAIN_AXIS: Vec3 = Vec3::Y;

impl HextonBundle {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            collider: Collider::triangle(
                HEXTON_VERTICES[0],
                HEXTON_VERTICES[1],
                HEXTON_VERTICES[2],
            ),
            collision_groups: CollisionGroups::new(
                collision_groups::DROIDS,
                collision_groups::DROIDS | collision_groups::LEVEL, /*  | collision_groups::PROJECTILES*/
            ),
            rigid_body: RigidBody::KinematicVelocityBased,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            name: Name::new(name),
            // weapon_direction: WeaponDirection { direction: Vec2::X },
            // weapon_state: default(),
            external_force: default(),
            // external_impulse: default(),
            // ship_input: default(),
            // ship_thruster: default(),
            attack_request: default(),
            // damping: default(),
            // read_mass_properties: default(),
            mass_properties: ColliderMassProperties::Density(1.0),
            spatial_bundle: default(),
            hexton_input: default(),
            character_controller: default(),
            controller_output: default(),
            gravity: default(),
        }
    }
}

fn apply_hexton_input_system(
    time: Res<Time>,
    mut query: Query<(
        &HextonInput,
        &mut KinematicCharacterController,
        &mut HextonGravity,
        &KinematicCharacterControllerOutput,
    )>,
) {
    for (input, mut controller, mut gravity, output) in &mut query {
        // let y = if !output.grounded { -1.0 } else { 0.0 };
        gravity.jump_timer.tick(time.delta());
        if input.jump && !gravity.jump && output.grounded {
            gravity.jump = true;
            gravity.jump_timer.reset();
            gravity.speed = 0.0;
        }

        let mut jump = 0.0;
        if gravity.jump && !gravity.jump_timer.finished() {
            jump = 1.0;
            info!("jump");
        }
        if gravity.jump && gravity.jump_timer.finished() {
            gravity.jump = false;
        }

        controller.translation = Some(Vec2::new(input.forward * 2.0, jump + gravity.speed));
        controller.autostep = Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(1.0),
            min_width: CharacterLength::Relative(0.0),
            include_dynamic_bodies: false,
        })
        // controller.snap_to_ground = Some(CharacterLength::Relative(0.1));
        // controller.
        // info!("forward: {}", input.forward);
    }
}

fn debug_hexton_input_system(
    mut query: Query<(&KinematicCharacterControllerOutput, &mut HextonGravity)>,
) {
    for (output, mut gravity) in &mut query {
        // info!("output: {:?}", output);
        if !output.grounded && !gravity.jump {
            gravity.speed -= 0.5;
            // external_force.force = Vec2::new(0.0, -9.81);
        } else {
            gravity.speed = 0.0;
            // external_force.force = Vec2::new(0.0, 0.0);
        }
    }
}

pub struct HextonPlugin;
impl Plugin for HextonPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(apply_hexton_input_system)
        .add_system(debug_hexton_input_system)
            // .add_system(ship_brake_maneuver_system.after(apply_ship_input_system))
            // .add_system(ship_thruster_system.after(ship_brake_maneuver_system))
            // // .add_system(ship_kinetic_debug_system.after(ship_thruster_system)) // can override other input
            // .add_system(ship_attack_system)
            ;
    }
}
