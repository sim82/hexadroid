use std::{f32::consts::TAU, ops::Range};

use crate::prelude::*;
use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics, RegisterDiagnostic},
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{prelude::Distribution, Rng};
use rand_distr::Normal;

// d'oh the only 128bin random number generator I found only produces binary numbers
pub const PARTICLE_COUNT: DiagnosticId = DiagnosticId::from_u128(0b00010001110010101001001110001101011100000101101010011100101100010111001010001001111001111010011011100100000001010100011100100011);
pub const NEW_PARTICLE_COUNT: DiagnosticId = DiagnosticId::from_u128(0b11110100000001001100011110101100010011111100101110011101001110011110111000111000101000100111101101111111101100001010011011101101);

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
    mut diagnostics: Diagnostics,
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
    diagnostics.add_measurement(NEW_PARTICLE_COUNT, || particle_batch.len() as f64);
    commands.spawn_batch(particle_batch);
}

fn evolve_particle_system(
    mut diagnostics: Diagnostics,
    mut query: Query<(&Particle, &mut Transform, &Despawn)>,
) {
    let mut num_particles = 0;
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
        num_particles += 1;
        //
    }
    diagnostics.add_measurement(PARTICLE_COUNT, || num_particles.into());
}
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleResources>()
            .register_diagnostic(
                Diagnostic::new(PARTICLE_COUNT, "particle count", 10).with_suffix("part"),
            )
            .register_diagnostic(
                Diagnostic::new(NEW_PARTICLE_COUNT, "new particle", 10).with_suffix("part/fr"),
            )
            .add_systems(Startup, init_particle_system)
            .add_systems(Update, (spawn_particle_system, evolve_particle_system));
    }
}
