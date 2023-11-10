use crate::{particle::ParticleDamping, prelude::*};
use bevy::prelude::*;
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*, shapes};
use bevy_rapier2d::prelude::*;
use rand_distr::Normal;

use crate::{collision_groups, prelude::ParticleSource, Despawn};
#[derive(Component)]
pub struct Projectile {
    pub owner: Entity,
}

#[derive(Bundle)]
pub struct KineticProjectileBundle {
    collider: Collider,
    collision_groups: CollisionGroups,
    // transform: Transform,
    rigid_body: RigidBody,
    velocity: Velocity,
    active_events: ActiveEvents,
    active_collision_types: ActiveCollisionTypes,
    projectile: Projectile,
    despawn: Despawn,
    mass_properies: ColliderMassProperties,
}

pub const PROJECTILE_SPEED: f32 = 800.0;

impl KineticProjectileBundle {
    pub fn with_direction(owner: Entity, /*translation: Vec3, */ direction: Vec2) -> Self {
        Self {
            collider: Collider::ball(20.0),
            collision_groups: CollisionGroups::new(
                collision_groups::PROJECTILES,
                collision_groups::DROIDS,
            ),
            rigid_body: RigidBody::Dynamic,
            velocity: Velocity::linear(direction * PROJECTILE_SPEED),
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_STATIC,
            projectile: Projectile { owner },
            despawn: Despawn::TimeToLive(10.0),
            mass_properies: ColliderMassProperties::Density(0.3),
        }
    }
}

pub fn kinetic_projectile_shape_bundle(translation: Vec3, direction: Vec2) -> ShapeBundle {
    let shape = shapes::Circle {
        radius: 10.0,
        ..default()
    };
    ShapeBundle {
        path: GeometryBuilder::build_as(
            &shape,
            // DrawMode::Stroke(StrokeMode::new(Color::GREEN, 10.0)),
            // fill_mode: bevy_prototype_lyon::draw::FillMode::color(Color::CYAN),
            // outline_mode: StrokeMode::new(Color::BLACK, 2.0),
            // },
        ),
        spatial: SpatialBundle {
            transform: Transform::from_translation(translation + (direction * 50.0).extend(0.0)),
            ..default()
        },
        ..default()
    }
}

#[derive(Bundle)]
pub struct WaveAttackBundle {
    pub spatial: SpatialBundle,
    pub particle_source: ParticleSource,
    pub despawn: Despawn,
}

impl WaveAttackBundle {
    pub fn wave_attack(translation: Vec3) -> WaveAttackBundle {
        WaveAttackBundle {
            spatial: SpatialBundle::from_transform(Transform::from_translation(translation)),
            particle_source: ParticleSource {
                rate: 200,
                direction: ParticleDirection::Uniform,
                speed_distr: Normal::new(800.0, 10.0).unwrap(),
                lifetime_distr: Normal::new(0.8, 0.5).unwrap(),
                velocity_offset: Vec2::default(),
                damping: ParticleDamping::None,
            },
            despawn: Despawn::FramesToLive(1),
        }
    }
}
