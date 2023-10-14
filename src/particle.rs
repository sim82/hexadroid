use std::f32::consts::TAU;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

pub enum ParticleDirection {
    DirectionalNormal { direction: Vec2, spread: f32 },
    Uniform,
}

#[derive(Component)]
pub struct ParticleSource {
    pub rate: u32,
    pub direction: ParticleDirection,
    pub speed: f32,
    pub speed_spread: f32,
    pub lifetime: f32,
    pub lifetime_spread: f32,
}

#[derive(Component, Default)]
struct Particle {
    pub initial_lifetime: f32,
}

#[derive(Bundle, Default)]
struct ParticleBundle {
    pub particle: Particle,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub despawn: Despawn,
}

fn spawn_particle_system(
    mut commands: Commands,
    source_query: Query<(&ParticleSource, &Transform)>,
) {
    let mut rng = rand::thread_rng();
    for (source, source_transform) in &source_query {
        for _ in 0..source.rate {
            let direction_vec = match source.direction {
                ParticleDirection::DirectionalNormal {
                    direction: _,
                    spread: _,
                } => todo!(),
                ParticleDirection::Uniform => Vec2::from_angle(rng.gen_range(0.0..TAU)),
            };

            let shape = shapes::Circle {
                radius: 0.5,
                ..default()
            };
            let shape = shapes::Rectangle {
                extents: Vec2::splat(0.5),
                ..default()
            };
            let speed = source.speed + rng.gen_range(-source.speed_spread..source.speed_spread);
            let lifetime =
                source.lifetime + rng.gen_range(-source.lifetime_spread..source.lifetime_spread);
            commands.spawn((
                ParticleBundle {
                    rigid_body: RigidBody::Dynamic,
                    velocity: Velocity::linear(direction_vec * speed),
                    despawn: Despawn::TimeToLive(lifetime),
                    particle: Particle {
                        initial_lifetime: lifetime,
                    },
                },
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    transform: *source_transform,
                    ..default()
                },
                default_stroke(COLORS[rng.gen_range(0..COLORS.len())]),
            ));
            //
        }
    }
}

fn evolve_particle_system(mut query: Query<(&Particle, &mut Transform, &Despawn)>) {
    for (particle, mut transform, despawn) in &mut query {
        let f = match despawn {
            Despawn::ThisFrame => continue,
            Despawn::TimeToLive(ttl) => {
                //
                ttl / particle.initial_lifetime
            }
        }
        .clamp(0.0, 1.0);

        transform.scale = Vec3::splat(f);
        //
    }
}
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_particle_system, evolve_particle_system));
    }
}
