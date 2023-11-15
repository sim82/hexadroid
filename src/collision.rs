use crate::{particle::ColorGenerator, prelude::*, weapon::Projectile, Despawn};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand_distr::Normal;

#[derive(Component, Debug, Copy, Clone)]
pub enum CollisionFxType {
    Spark,
}
fn display_events_system(mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.read() {
        info!("Received collision event: {:?}", collision_event);
    }
}

fn projectile_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    projectile_query: Query<Entity, With<Projectile>>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(a, b, _) => {
                let projectile = projectile_query
                    .get(*a)
                    .or_else(|_| projectile_query.get(*b));

                if let Ok(projectile) = projectile {
                    commands
                        .entity(projectile)
                        .insert(ParticleSource {
                            rate: 200,
                            direction: ParticleDirection::Uniform,
                            speed_distr: Normal::new(200.0, 90.0).unwrap(),
                            lifetime_distr: Normal::new(0.8, 0.5).unwrap(),
                            velocity_offset: Vec2::default(),
                            damping: default(),
                            initial_offset: 0.0,
                            color_generator: ColorGenerator::Static(7),
                            // color_generator: ColorGenerator::Random,
                        })
                        .insert(Despawn::TimeToLive(0.1))
                        // don't register more Projectile collisions in the next frames
                        .remove::<Projectile>();
                }
            }
            CollisionEvent::Stopped(_, _, _) => (),
        }
    }
}

fn collision_fx_system(
    _commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    collision_fx_query: Query<&CollisionFxType>,
) {
    for collision_event in collision_events.read() {
        info!("collision event: {collision_event:?}");
        match collision_event {
            CollisionEvent::Started(a, b, _) => {
                if collision_fx_query.contains(*a) && collision_fx_query.contains(*b) {
                    info!("spark {a:?} {b:?}");
                }
            }
            CollisionEvent::Stopped(_, _, _) => (),
        }
    }
}
pub struct CollisionPlugin;
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, display_events_system)
            .add_systems(Update, (projectile_collision_system, collision_fx_system));
    }
}
