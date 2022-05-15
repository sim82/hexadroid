use bevy::prelude::*;
use hexagon_tiles::{
    layout::{Layout, LAYOUT_ORIENTATION_POINTY},
    point::Point,
};

pub mod droid;
pub mod input;

pub const HEX_LAYOUT: Layout = Layout {
    orientation: LAYOUT_ORIENTATION_POINTY,
    size: Point { x: 64.0, y: 64.0 },
    origin: Point { x: 0.0, y: 0.0 },
};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub enum Despawn {
    ThisFrame,
    TimeToLive(f32),
}

pub fn despawn_reaper_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Despawn)>,
) {
    for (entity, mut despawn) in query.iter_mut() {
        let despawn = match *despawn {
            Despawn::ThisFrame => true,
            Despawn::TimeToLive(ref mut ttl) => {
                *ttl -= time.delta_seconds();
                *ttl <= 0.0
            }
        };
        if despawn {
            info!("despawn {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub mod collision {
    use crate::{droid::Projectile, Despawn};
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
                        commands.entity(projectile).insert(Despawn::ThisFrame);
                    }
                }
                CollisionEvent::Stopped(_, _, _) => (),
            }
        }
    }

    pub struct CollisionPlugin;
    impl Plugin for CollisionPlugin {
        fn build(&self, app: &mut App) {
            app.add_system(display_events_system)
                .add_system(projectile_collision_system);
        }
    }
}
pub struct DefaultPlugin;
impl Plugin for DefaultPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(despawn_reaper_system);
    }
}

pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        use bevy_rapier2d::prelude::*;

        group.add(DefaultPlugin);

        // bevy_rapier plugins
        group
            .add(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
            .add(RapierDebugRenderPlugin::default());

        // egui plugins
        #[cfg(feature = "inspector")]
        {
            group.add(bevy_inspector_egui::WorldInspectorPlugin::new());
        }

        group
            .add(DefaultPlugin)
            .add(input::InputPlugin)
            .add(droid::DroidPlugin)
            .add(collision::CollisionPlugin);
    }
}
