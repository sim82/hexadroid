use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::input::InputTarget;

const STOP_CUTOFF: f32 = 0.5;
const STOP_MULTIPLIER: f32 = -5.0;
const FORCE_MULTIPLIER: f32 = 10.0;

#[derive(Component)]
pub struct GroundFriction;

#[derive(Component, Default)]
pub struct WeaponDirection {
    direction: Vec2,
}

fn droid_stop_system(mut query: Query<(&mut Velocity, &mut ExternalForce), With<GroundFriction>>) {
    for (mut velocity, mut external_force) in query.iter_mut() {
        info!("vel: {}", velocity.linvel);

        if velocity.linvel.length() <= STOP_CUTOFF {
            velocity.linvel = Vec2::ZERO;
            continue;
        }

        external_force.force = STOP_MULTIPLIER * velocity.linvel;
    }
}

fn droid_apply_direction_system(
    mut query: Query<(&mut ExternalForce, &InputTarget, &mut WeaponDirection)>,
) {
    for (mut external_force, input_target, mut weapon_direction) in query.iter_mut() {
        info!("force: {}", external_force.force);
        if input_target.direction.length() > f32::EPSILON {
            external_force.force = FORCE_MULTIPLIER * input_target.direction;
            weapon_direction.direction = input_target.direction;
        }
    }
}

#[derive(Bundle)]
pub struct DroidBundle {
    pub collider: Collider,
    pub transform: Transform,
    pub external_force: ExternalForce,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    pub friction: Friction,
    pub restitution: Restitution,
    pub velocity: Velocity,
    pub name: Name,
    pub ground_friction: GroundFriction,
    pub weapon_direction: WeaponDirection,
}

impl Default for DroidBundle {
    fn default() -> Self {
        Self {
            collider: Collider::ball(32.0),
            transform: Transform::from_xyz(200.0, 0.0, 0.0),
            external_force: ExternalForce::default(),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            friction: Friction {
                coefficient: 0.5,
                ..default()
            },
            restitution: Restitution {
                coefficient: 1.0,
                ..default()
            },
            velocity: Velocity::default(),
            name: Name::new("droid"),
            weapon_direction: WeaponDirection::default(),
            ground_friction: GroundFriction,
        }
    }
}

pub struct DroidPlugin;

impl Plugin for DroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(droid_stop_system)
            .add_system(droid_apply_direction_system.after(droid_stop_system));
    }
}
