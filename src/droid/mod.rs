use std::borrow::Cow;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::input::InputTarget;

use self::{ai::PrimaryEnemy, weapon::kinetic_projectile_shape_bundle};

pub mod weapon;

pub mod ai;

const STOP_CUTOFF: f32 = 0.5;
const STOP_MULTIPLIER: f32 = -15.0;
const FORCE_MULTIPLIER: f32 = 4000.0;
const IMPULSE_MULTIPLIER: f32 = 80.0;
const RELOAD_TIMEOUT: f32 = 0.5;

#[derive(Component)]
pub struct GroundFriction;

#[derive(Component, Default)]
pub struct WeaponDirection {
    direction: Vec2,
}

#[derive(Component, Default)]
pub struct WeaponState {
    reload_timeout: f32,
}

#[derive(Component, Default)]
pub struct TargetDirection {
    pub direction: Vec2,
}

#[derive(Component, Default)]
pub struct AttackRequest {
    pub primary_attack: bool,
}

fn droid_stop_system(mut query: Query<(&mut Velocity, &mut ExternalForce), With<GroundFriction>>) {
    for (mut velocity, mut external_force) in query.iter_mut() {
        // info!("vel: {}", velocity.linvel);

        if velocity.linvel.length() <= STOP_CUTOFF {
            velocity.linvel = Vec2::ZERO;
            continue;
        }

        external_force.force = STOP_MULTIPLIER * velocity.linvel;
    }
}

fn droid_apply_direction_system(
    mut query: Query<(&mut ExternalImpulse, &TargetDirection, &mut WeaponDirection)>,
) {
    for (mut external_impulse, target_direction, mut weapon_direction) in query.iter_mut() {
        if target_direction.direction.length() > f32::EPSILON {
            external_impulse.impulse = IMPULSE_MULTIPLIER * target_direction.direction;
            weapon_direction.direction = target_direction.direction;
        }
    }
}

fn droid_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &Transform,
        &AttackRequest,
        &mut WeaponState,
        &WeaponDirection,
    )>,
) {
    for (
        entity,
        Transform { translation, .. },
        AttackRequest { primary_attack },
        mut weapon_state,
        weapon_direction,
    ) in query.iter_mut()
    {
        weapon_state.reload_timeout = (weapon_state.reload_timeout - time.delta_seconds()).max(0.0);
        if !primary_attack || weapon_state.reload_timeout > f32::EPSILON {
            continue;
        }
        weapon_state.reload_timeout = RELOAD_TIMEOUT;
        commands
            .spawn_bundle(weapon::KineticProjectileBundle::with_direction(
                entity,
                // *translation,
                weapon_direction.direction,
            ))
            .insert_bundle(kinetic_projectile_shape_bundle(
                *translation,
                weapon_direction.direction,
            ));
    }
}

#[derive(Bundle)]
pub struct DroidBundle {
    pub collider: Collider,
    // pub transform: Transform,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    pub friction: Friction,
    pub restitution: Restitution,
    pub velocity: Velocity,
    pub name: Name,
    pub ground_friction: GroundFriction,
    pub weapon_direction: WeaponDirection,
    pub weapon_state: WeaponState,
    pub target_direction: TargetDirection,
    pub attack_request: AttackRequest,
    pub damping: Damping,
}

impl DroidBundle {
    pub fn with_name(/*translation: Vec2, */ name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            collider: Collider::ball(28.0),
            // transform: Transform::from_xyz(translation.x, translation.y, 0.0),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            friction: Friction {
                coefficient: 0.5,
                ..default()
            },
            restitution: Restitution {
                coefficient: 0.3,
                ..default()
            },
            velocity: Velocity::default(),
            name: Name::new(name),
            ground_friction: GroundFriction,
            weapon_direction: WeaponDirection { direction: Vec2::X },
            weapon_state: default(),
            external_force: default(),
            external_impulse: default(),
            target_direction: default(),
            attack_request: default(),
            damping: Damping {
                linear_damping: 5.0,
                ..default()
            },
        }
    }
}
pub struct DroidPlugin;

impl Plugin for DroidPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_system(droid_stop_system)
            .add_system(droid_apply_direction_system) //.after(droid_stop_system))
            .add_system(droid_attack_system);
    }
}
