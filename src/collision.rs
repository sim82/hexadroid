use crate::{droid::weapon::Projectile, prelude::*, Despawn};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

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
                            rate: 100,
                            direction: ParticleDirection::Uniform,
                            speed: 100.0,
                            speed_spread: 80.0,
                            lifetime: 1.0,
                            lifetime_spread: 0.5,
                        })
                        .insert(Despawn::TimeToLive(0.2))
                        // don't register more Projectile collisions in the next frames
                        .remove::<Projectile>();
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
            .add_systems(Update, projectile_collision_system);
    }
}
