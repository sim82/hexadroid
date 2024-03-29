use self::ai::{EnemyEvaluation, PredictedHit, PrimaryEnemy};
use crate::collision_groups;
use crate::weapon::WeaponTarget;
use crate::{
    collision::CollisionFxType, prelude::*, weapon, weapon::kinetic_projectile_shape_bundle,
};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::Stroke;
use bevy_rapier2d::prelude::*;
use rand::Rng;
use std::{borrow::Cow, time::Duration};

pub mod ai;

// const STOP_CUTOFF: f32 = 0.5;
// const STOP_MULTIPLIER: f32 = -15.0;
// const FORCE_MULTIPLIER: f32 = 4000.0;
pub const IMPULSE_MULTIPLIER: f32 = 8.0;
pub const RELOAD_TIMEOUT: f32 = 1.0;
#[derive(Component, Default)]
pub struct AiMarker;

#[derive(Component, Default)]
pub struct DroidMarker;

#[derive(Component)]
pub struct GroundFriction;

#[derive(Component, Default)]
pub struct WeaponDirection {
    direction: Vec2,
}

#[derive(Component, Default)]
pub struct WeaponState {
    pub reload_timeout: f32,
}

#[derive(Component, Default)]
pub struct TargetDirection {
    pub direction: Vec2,
}

#[derive(Component, Default)]
pub struct AttackRequest {
    pub primary_attack: bool,
}

#[derive(Component, Default)]
pub struct MovementStats {
    pub idle_duration: Duration,
}
// fn droid_stop_system(mut query: Query<(&mut Velocity, &mut ExternalForce), With<GroundFriction>>) {
//     for (mut velocity, mut external_force) in query.iter_mut() {
//         // info!("vel: {}", velocity.linvel);

//         if velocity.linvel.length() <= STOP_CUTOFF {
//             velocity.linvel = Vec2::ZERO;
//             continue;
//         }

//         external_force.force = STOP_MULTIPLIER * velocity.linvel;
//     }
// }

fn droid_apply_direction_system(
    time: Res<Time>,
    mut query: Query<
        (
            &mut ExternalImpulse,
            &TargetDirection,
            &mut WeaponDirection,
            &mut MovementStats,
        ),
        Without<DroidOverloadMarker>,
    >,
) {
    for (mut external_impulse, target_direction, mut weapon_direction, mut movement_stats) in
        query.iter_mut()
    {
        if target_direction.direction.length() > f32::EPSILON {
            external_impulse.impulse = IMPULSE_MULTIPLIER * target_direction.direction;
            weapon_direction.direction = target_direction.direction;
            movement_stats.idle_duration = default();
        } else {
            movement_stats.idle_duration += time.delta();
        }
    }
}

fn droid_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &AttackRequest,
            &mut WeaponState,
            &WeaponDirection,
        ),
        Without<DroidOverloadMarker>,
    >,
) {
    for (
        entity,
        Transform { translation, .. },
        AttackRequest { primary_attack },
        mut weapon_state,
        weapon_direction,
    ) in query.iter_mut()
    {
        weapon_state.reload_timeout = (weapon_state.reload_timeout - time.delta_seconds()).max(0.0);
        if !primary_attack || weapon_state.reload_timeout > f32::EPSILON {
            continue;
        }
        weapon_state.reload_timeout = RELOAD_TIMEOUT;
        commands
            .spawn(weapon::KineticProjectileBundle::with_direction(
                entity,
                // *translation,
                weapon_direction.direction,
            ))
            .insert(kinetic_projectile_shape_bundle(
                *translation,
                weapon_direction.direction,
            ))
            .insert(Stroke::new(GREEN_HDR, 10.0));
    }
}

fn droid_overload_system(
    mut commands: Commands,
    query: Query<(Entity, &DroidHealth), Without<DroidOverloadMarker>>,
    mut query_overload: Query<
        (Entity, &mut DroidHealth, &mut Transform),
        With<DroidOverloadMarker>,
    >,
) {
    //
    for (entity, health) in &query {
        if health.emp_load >= 1.0 {
            commands.entity(entity).insert(DroidOverloadMarker);
        }
    }
    for (entity, mut health, mut transform) in &mut query_overload {
        if health.emp_load <= 0.5 {
            commands.entity(entity).remove::<DroidOverloadMarker>();
            transform.scale = Vec3::ONE;
        } else {
            let mut rng = rand::thread_rng();
            transform.scale = Vec3::splat(rng.gen_range(0.95..1.05));
        }
        health.emp_load = (health.emp_load - 0.005).max(0.0);
    }
}

#[derive(Bundle)]
pub struct DroidBundle {
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    // pub transform: Transform,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    // pub friction: Friction,
    // pub restitution: Restitution,
    pub velocity: Velocity,
    pub name: Name,
    // pub ground_friction: GroundFriction,
    pub weapon_direction: WeaponDirection,
    pub weapon_state: WeaponState,
    pub target_direction: TargetDirection,
    pub attack_request: AttackRequest,
    pub movement_stats: MovementStats,
    pub damping: Damping,
    pub mass_properties: ColliderMassProperties,
    // #[bundle]
    pub spatial_bundle: SpatialBundle,
    pub collision_fx: CollisionFxType,
    pub active_events: ActiveEvents,
    pub active_collision_types: ActiveCollisionTypes,
    pub weapon_target: WeaponTarget,
    pub droid_marker: DroidMarker,
    pub health: DroidHealth,
}

impl DroidBundle {
    pub fn new(
        /*translation: Vec2, */ name: impl Into<Cow<'static, str>>,
        gravity: bool,
    ) -> Self {
        let locked_axes;
        let damping;
        if gravity {
            locked_axes = LockedAxes::default();
            damping = Damping::default();
        } else {
            locked_axes = LockedAxes::ROTATION_LOCKED;
            damping = Damping {
                linear_damping: 5.0,
                ..default()
            };
        }

        Self {
            collider: Collider::ball(28.0),
            collision_groups: CollisionGroups::new(
                collision_groups::DROIDS,
                collision_groups::DROIDS | collision_groups::PROJECTILES | collision_groups::LEVEL,
            ),
            rigid_body: RigidBody::Dynamic,
            locked_axes,
            velocity: Velocity::default(),
            name: Name::new(name),
            weapon_direction: WeaponDirection { direction: Vec2::X },
            weapon_state: default(),
            external_force: default(),
            external_impulse: default(),
            target_direction: default(),
            attack_request: default(),
            movement_stats: default(),
            damping,
            mass_properties: ColliderMassProperties::Density(1.0),
            spatial_bundle: default(),
            collision_fx: CollisionFxType::Spark,
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_STATIC,
            weapon_target: default(),
            droid_marker: default(),
            health: default(),
        }
    }
}

#[derive(Component, Default)]
pub struct DroidHealth {
    pub emp_load: f32,
}
#[derive(Component)]
pub struct DroidOverloadMarker;

#[derive(Bundle)]
pub struct AiDroidBundle {
    predicted_hit: PredictedHit,
    enemy_evaluation: EnemyEvaluation,
    primary_enemy: PrimaryEnemy,
    spatial_bundle: SpatialBundle,
    ai_marker: AiMarker,
}

impl AiDroidBundle {
    pub fn with_enemy(enemy: Entity) -> Self {
        Self {
            predicted_hit: PredictedHit::default(),
            enemy_evaluation: EnemyEvaluation::default(),
            primary_enemy: PrimaryEnemy { enemy },
            spatial_bundle: default(),
            ai_marker: default(),
        }
    }
}

pub struct DroidPlugin;

impl Plugin for DroidPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_system(droid_stop_system)
            .add_systems(
                Update,
                (
                    droid_apply_direction_system, //.after(droid_stop_system))
                    droid_attack_system,
                    droid_overload_system,
                )
                    .run_if(in_state(GameState::Game)),
            );
    }
}
