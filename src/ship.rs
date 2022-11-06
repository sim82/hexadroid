use std::borrow::Cow;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_rapier2d::{na::Quaternion, prelude::*};

use crate::{
    collision_groups,
    droid::{
        weapon::{self, kinetic_projectile_shape_bundle},
        AttackRequest, WeaponState, RELOAD_TIMEOUT,
    },
};

#[derive(Component, Default)]
pub struct ShipInput {
    pub rot: f32,
    pub thrust: f32,
    pub brake: f32,
}

#[derive(Component, Default)]
pub struct ShipThruster {
    pub forward: f32,
    pub rot: f32,
}

#[derive(Bundle)]
pub struct ShipBundle {
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    // pub transform: Transform,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    // pub friction: Friction,
    // pub restitution: Restitution,
    pub velocity: Velocity,
    pub name: Name,
    // pub ground_friction: GroundFriction,
    // pub weapon_direction: WeaponDirection,
    pub weapon_state: WeaponState,
    pub ship_input: ShipInput,
    pub ship_thruster: ShipThruster,

    pub attack_request: AttackRequest,
    pub damping: Damping,
    // pub mass_properties: ColliderMassProperties,
    #[bundle]
    pub spatial_bundle: SpatialBundle,
}

pub const SHIP_VERTICES: [Vec2; 3] = [
    Vec2::new(0.0, 30.0),
    Vec2::new(-15.0, -30.0),
    Vec2::new(15.0, -30.0),
];

pub const SHIP_MAIN_AXIS: Vec3 = Vec3::Y;

impl ShipBundle {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            collider: Collider::triangle(SHIP_VERTICES[0], SHIP_VERTICES[1], SHIP_VERTICES[2]),
            collision_groups: CollisionGroups::new(
                collision_groups::DROIDS,
                collision_groups::DROIDS, /*  | collision_groups::PROJECTILES*/
            ),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::empty(),
            velocity: Velocity::default(),
            name: Name::new(name),
            // weapon_direction: WeaponDirection { direction: Vec2::X },
            weapon_state: default(),
            external_force: default(),
            external_impulse: default(),
            ship_input: default(),
            ship_thruster: default(),
            attack_request: default(),
            damping: default(),
            // mass_properties: ColliderMassProperties::Density(1.0),
            spatial_bundle: default(),
        }
    }
}

pub fn apply_ship_input_system(
    mut ship_query: Query<(&ShipInput, &Transform, &mut ExternalImpulse, &mut Damping)>,
) {
    for (ship_input, transform, mut extrnal_impulse, mut damping) in &mut ship_query {
        // info!( "{} {}", )
        extrnal_impulse.torque_impulse = ship_input.rot * -0.001;
        let forward = transform.rotation * (SHIP_MAIN_AXIS * 0.5); // Vec3::new(0.0, 0.5, 0.0);
        extrnal_impulse.impulse = forward.xy() * ship_input.thrust;
        damping.angular_damping = ship_input.thrust * 2.0;
        // damping.linear_damping = ship_input.brake;
    }
}

pub fn ship_brake_maneuver_system(
    mut query: Query<(&ShipInput, &Transform, &Velocity, &mut ExternalImpulse)>,
) {
    for (ship_input, transform, velocity, mut external_impulse) in &mut query {
        let Some(movement_dir) = velocity.linvel.try_normalize() else {
             continue;
         };

        // let movement_dir = SHIP_MAIN_AXIS.xy();
        let ship_dir = (transform.rotation * SHIP_MAIN_AXIS).xy();
        let angle = ship_dir.angle_between(movement_dir);
        // let x = ship_dir.dot(movement_dir) * ship_dir.perp_dot(movement_dir).signum();
        let diff = 1.0 - angle.abs() / (std::f32::consts::PI * 4.0);

        if ship_input.brake > f32::EPSILON {
            if (angle.signum() == velocity.angvel.signum() || velocity.angvel.abs() < 2.0) {
                external_impulse.torque_impulse = (diff * -0.002) * angle.signum();
                // todo!();
            }
            if diff < 0.8 {
                let forward = (transform.rotation * SHIP_MAIN_AXIS).xy();
                external_impulse.impulse = forward;
            }
            info!("diff {}", diff);
        }
    }
}

fn ship_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &AttackRequest, &mut WeaponState), With<ShipInput>>,
) {
    for (entity, transform, AttackRequest { primary_attack }, mut weapon_state) in query.iter_mut()
    {
        weapon_state.reload_timeout = (weapon_state.reload_timeout - time.delta_seconds()).max(0.0);
        if !primary_attack || weapon_state.reload_timeout > f32::EPSILON {
            continue;
        }
        weapon_state.reload_timeout = RELOAD_TIMEOUT;
        let direction = (transform.rotation * SHIP_MAIN_AXIS).xy();
        commands
            .spawn_bundle(weapon::KineticProjectileBundle::with_direction(
                entity, // *translation,
                direction,
            ))
            .insert_bundle(kinetic_projectile_shape_bundle(
                transform.translation,
                direction,
            ));
    }
}

pub struct ShipPlugin;
impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(apply_ship_input_system)
            .add_system(ship_brake_maneuver_system.after(apply_ship_input_system)) // can override other input
            .add_system(ship_attack_system);
    }
}
