use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::Despawn;
#[derive(Component)]
pub struct Projectile {
    pub owner: Entity,
}

#[derive(Bundle)]
pub struct KineticProjectileBundle {
    collider: Collider,
    transform: Transform,
    rigid_body: RigidBody,
    active_events: ActiveEvents,
    velocity: Velocity,
    active_collision_types: ActiveCollisionTypes,
    projectile: Projectile,
    despawn: Despawn,
}

impl KineticProjectileBundle {
    pub fn with_direction(owner: Entity, translation: Vec3, direction: Vec2) -> Self {
        Self {
            collider: Collider::ball(10.0),
            transform: Transform::from_translation(translation + (direction * 100.0).extend(0.0)),
            rigid_body: RigidBody::Dynamic,
            velocity: Velocity::linear(direction * 400.0),
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_STATIC,
            projectile: Projectile { owner },
            despawn: Despawn::TimeToLive(10.0),
        }
    }
}
