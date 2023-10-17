use std::{f32::consts::TAU, ops::Range};

use crate::prelude::*;
use bevy::{
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{prelude::Distribution, Rng};
use rand_distr::Normal;

#[derive(Resource, Default)]
pub struct ParticleResources {
    pub mesh: Mesh2dHandle,
    pub materials: Vec<Handle<ColorMaterial>>,
}

pub enum ParticleDirection {
    DirectionalNormal { direction: f32, std_dev: f32 },
    Uniform,
}

#[derive(Component)]
pub struct ParticleSource {
    pub rate: u32,
    pub direction: ParticleDirection,
    pub speed_distr: Normal<f32>,
    pub lifetime_distr: Normal<f32>,
    pub velocity_offset: Vec2,
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

fn init_particle_system(
    mut commands: Commands,
    mut res: ResMut<ParticleResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let q = shape::Quad::new(Vec2::splat(2.5));
    // let q = shape::Box::new(100.0, 100.0, 100.0);
    res.mesh = meshes.add(q.into()).into();
    for color in &COLORS {
        res.materials.push(materials.add(ColorMaterial {
            color: *color,
            ..default()
        }));
    }
}
fn spawn_particle_system(
    mut commands: Commands,
    res: Res<ParticleResources>,
    source_query: Query<(&ParticleSource, &GlobalTransform)>,
) {
    let mut rng = rand::thread_rng();
    let mut particle_batch = Vec::new();
    for (source, source_transform) in &source_query {
        particle_batch.reserve(source.rate as usize);
        let material = &res.materials[rng.gen_range(0..res.materials.len())];
        let transform = source_transform.compute_transform();
        for _ in 0..source.rate {
            let direction_vec = match source.direction {
                ParticleDirection::DirectionalNormal { direction, std_dev } => {
                    let direction = direction; //.angle_between(Vec2::X);
                    let distr = Normal::new(direction, std_dev).unwrap();
                    let dir = distr.sample(&mut rng);
                    Vec2::from_angle(dir)
                }
                ParticleDirection::Uniform => Vec2::from_angle(rng.gen_range(0.0..TAU)),
            };

            let speed = source.speed_distr.sample(&mut rng).max(0.0);
            let lifetime = source.lifetime_distr.sample(&mut rng).max(0.0);

            particle_batch.push((
                ParticleBundle {
                    rigid_body: RigidBody::Dynamic,
                    velocity: Velocity::linear(source.velocity_offset + direction_vec * speed),
                    despawn: Despawn::TimeToLive(lifetime),
                    particle: Particle {
                        initial_lifetime: lifetime,
                    },
                },
                MaterialMesh2dBundle {
                    mesh: res.mesh.clone(),
                    material: material.clone(),
                    transform,
                    ..default()
                },
            ));
        }
    }
    commands.spawn_batch(particle_batch);
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
        app.init_resource::<ParticleResources>()
            .add_systems(Startup, init_particle_system)
            .add_systems(Update, (spawn_particle_system, evolve_particle_system));
    }
}
