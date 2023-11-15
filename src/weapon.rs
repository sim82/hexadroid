use crate::{
    particle::{ColorGenerator, ParticleDamping},
    prelude::*,
};
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

#[derive(Component)]
pub struct WaveAttack;

#[derive(Bundle)]
pub struct WaveAttackBundle {
    pub spatial: SpatialBundle,
    pub particle_source: ParticleSource,
    pub despawn: Despawn,
    pub wave_attack: WaveAttack,
}

impl WaveAttackBundle {
    pub fn wave_attack(translation: Vec3) -> WaveAttackBundle {
        WaveAttackBundle {
            spatial: SpatialBundle::from_transform(Transform::from_translation(translation)),
            particle_source: ParticleSource {
                rate: 400,
                direction: ParticleDirection::Uniform,
                speed_distr: Normal::new(1800.0, 10.0).unwrap(),
                lifetime_distr: Normal::new(0.4, 0.01).unwrap(),
                velocity_offset: Vec2::default(),
                damping: ParticleDamping::None,
                initial_offset: 0.0,
                // color_generator: ColorGenerator::Static(2),
                color_generator: ColorGenerator::Random,
            },
            despawn: Despawn::FramesToLive(1),
            wave_attack: WaveAttack,
        }
    }
}

#[derive(Component)]
pub struct WaveAttackProxy {
    pub target: Entity,
    pub timeout: f32,
    pub source_dir: Vec2,
}

pub fn wave_attack_spawn_proxies(
    mut commands: Commands,
    query: Query<&Transform, Added<WaveAttack>>,
    target_query: Query<(Entity, &Transform), With<WeaponTarget>>,
) {
    for transform in &query {
        for (target, target_transform) in &target_query {
            let d = transform.translation - target_transform.translation;
            let dist = d.length();
            let timeout = dist / 1800.0;
            commands.spawn_empty().insert(WaveAttackProxy {
                target,
                timeout,
                source_dir: d.xy().normalize_or_zero(),
            });
        }
    }
}
pub fn wave_attack_proxy_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut WaveAttackProxy)>,
    target_query: Query<&Transform, With<WeaponTarget>>,
) {
    for (proxy_entity, mut proxy) in &mut query {
        proxy.timeout -= time.delta_seconds();
        if proxy.timeout <= 0.0 {
            let Ok(target_transform) = target_query.get(proxy.target) else {
                continue;
            };
            //
            // let mut fx_transform = *target_transform;
            // fx_transform.translation += (proxy.source_dir * 50.0).extend(0.0);
            commands
                .spawn(SpatialBundle::from_transform(*target_transform))
                .insert(ParticleSource {
                    rate: 200,
                    direction: ParticleDirection::DirectionalNormal {
                        direction: -proxy.source_dir.angle_between(Vec2::X),
                        std_dev: 0.3,
                    },
                    speed_distr: Normal::new(100.0, 10.0).unwrap(),
                    lifetime_distr: Normal::new(0.4, 0.05).unwrap(),
                    velocity_offset: Vec2::default(),
                    damping: default(),
                    initial_offset: 0.45,
                    color_generator: ColorGenerator::Static(7),
                })
                .insert(Despawn::FramesToLive(1));
            commands.entity(proxy_entity).insert(Despawn::ThisFrame);
        }
    }
}
#[derive(Component)]
pub struct WeaponTarget {
    pub kinetic_projectile: bool,
    pub wave_attack: bool,
}

impl Default for WeaponTarget {
    fn default() -> Self {
        Self {
            kinetic_projectile: true,
            wave_attack: true,
        }
    }
}
pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_system(droid_stop_system)
            .add_systems(Update, wave_attack_proxy_update) //.after(droid_stop_system))
            .add_systems(Update, wave_attack_spawn_proxies);
    }
}
