use crate::{droid::weapon::Projectile, prelude::*, Despawn};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component, Debug, Copy, Clone)]
pub enum CollisionFxType {
    Spark,
}
fn display_events_system(mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.iter() {
        info!("Received collision event: {:?}", collision_event);
    }
}

fn projectile_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    projectile_query: Query<Entity, With<Projectile>>,
) {
    for collision_event in collision_events.iter() {
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
                            speed: 200.0,
                            speed_spread: 180.0,
                            lifetime: 0.80,
                            lifetime_spread: 0.5,
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
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    collision_fx_query: Query<&CollisionFxType>,
) {
    for collision_event in collision_events.iter() {
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
