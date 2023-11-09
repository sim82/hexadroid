use crate::prelude::*;
use crate::{
    collision_groups,
    droid::{
        weapon::{self, kinetic_projectile_shape_bundle, PROJECTILE_SPEED},
        AttackRequest, WeaponState, RELOAD_TIMEOUT,
    },
};
use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_prototype_lyon::prelude::Stroke;
use bevy_rapier2d::prelude::*;
use rand_distr::Normal;
use std::borrow::Cow;

#[derive(Component, Default)]
pub struct ShipInput {
    pub rot: f32,
    pub thrust: f32,
    pub brake: f32,
}

#[derive(Component, Default)]
pub struct ShipThruster {
    pub forward: f32,
    pub rot: f32,
    pub rot_damping: bool,
}

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct ShipBrakeManeuver {
    pub direction: Vec2,
}

impl ShipThruster {
    pub fn apply_clamping(&mut self) {
        self.forward = self.forward.clamp(0.0, 1.0);
        self.rot = self.rot.clamp(-1.0, 1.0);
    }
}

// marker component for the particle spawner attached to ships
#[derive(Component)]
pub struct ShipThrusterParticleSpawner;

#[derive(Bundle)]
pub struct ShipBundle {
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
    // pub weapon_direction: WeaponDirection,
    pub weapon_state: WeaponState,
    pub ship_input: ShipInput,
    pub ship_thruster: ShipThruster,

    pub attack_request: AttackRequest,
    pub damping: Damping,
    pub read_mass_properties: ReadMassProperties,
    // pub mass_properties: ColliderMassProperties,
    // #[bundle]
    pub spatial_bundle: SpatialBundle,
}

pub const SHIP_VERTICES: [Vec2; 3] = [
    Vec2::new(0.0, 30.0),
    Vec2::new(-15.0, -30.0),
    Vec2::new(15.0, -30.0),
];

pub const SHIP_MAIN_AXIS: Vec3 = Vec3::Y;

impl ShipBundle {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            collider: Collider::triangle(SHIP_VERTICES[0], SHIP_VERTICES[1], SHIP_VERTICES[2]),
            collision_groups: CollisionGroups::new(
                collision_groups::DROIDS,
                collision_groups::DROIDS | collision_groups::LEVEL, /*  | collision_groups::PROJECTILES*/
            ),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::empty(),
            velocity: Velocity::default(),
            name: Name::new(name),
            // weapon_direction: WeaponDirection { direction: Vec2::X },
            weapon_state: default(),
            external_force: default(),
            external_impulse: default(),
            ship_input: default(),
            ship_thruster: default(),
            attack_request: default(),
            damping: default(),
            read_mass_properties: default(),
            // mass_properties: ColliderMassProperties::Density(1.0),
            spatial_bundle: default(),
        }
    }
}

pub fn apply_ship_input_system(
    mut commands: Commands,
    mut ship_query: Query<(Entity, &ShipInput, &Velocity, &mut ShipThruster)>,
    brake_query: Query<&ShipBrakeManeuver>,
) {
    for (entity, ship_input, velocity, mut thruster) in &mut ship_query {
        thruster.forward = ship_input.thrust;
        thruster.rot_damping = ship_input.thrust > f32::EPSILON;

        thruster.rot = ship_input.rot * 0.5;

        if ship_input.brake > f32::EPSILON && !brake_query.contains(entity) {
            commands.entity(entity).insert(ShipBrakeManeuver {
                direction: velocity.linvel,
            });
        } else if ship_input.brake < f32::EPSILON {
            commands.entity(entity).remove::<ShipBrakeManeuver>();
        }
    }
}
pub fn ship_attach_thruster_particle_spawner_system(
    mut commands: Commands,
    query: Query<Entity, Added<ShipThruster>>,
) {
    for entity in &query {
        commands.entity(entity).with_children(|commands| {
            //
            commands.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(SHIP_MAIN_AXIS * -30.0),
                    ..default()
                },
                ParticleSource {
                    lifetime_distr: Normal::new(0.8, 0.5).unwrap(),
                    speed_distr: Normal::new(200.0, 90.0).unwrap(),
                    rate: 0,
                    direction: ParticleDirection::Uniform,
                    velocity_offset: Vec2::default(),
                },
                ShipThrusterParticleSpawner,
            ));
        });
    }
}
pub fn ship_brake_maneuver_system(
    mut query: Query<(
        &ShipInput,
        &Transform,
        &Velocity,
        &ShipBrakeManeuver,
        &mut ShipThruster,
    )>,
) {
    for (ship_input, transform, velocity, brake_maneuver, mut thruster) in &mut query {
        let (Some(maneuver_dir), Some(cur_dir)) = (
            brake_maneuver.direction.try_normalize(),
            velocity.linvel.try_normalize(),
        ) else {
            thruster.rot_damping |= ship_input.brake > f32::EPSILON;
            continue;
        };
        let ship_dir = (transform.rotation * SHIP_MAIN_AXIS).xy();
        let angle = ship_dir.angle_between(maneuver_dir);
        let diff = 1.0 - angle.abs() / (std::f32::consts::PI * 4.0);
        if ship_input.brake > f32::EPSILON {
            let dot = maneuver_dir.dot(cur_dir);
            if diff < 0.8 && dot > 0.0 {
                thruster.forward = 1.0;
                // thruster.rot_damping = true;
            }
            if angle.signum() == velocity.angvel.signum() || velocity.angvel.abs() < 2.0 {
                thruster.rot = diff * angle.signum()
            }
            // info!("diff {}", diff);
        }
    }
}

// fn ship_thruster_system(
//     mut query: Query<(
//         &mut ShipThruster,
//         &Transform,
//         &mut ExternalImpulse,
//         &mut Damping,
//     )>,
// ) {
//     const ROT_IMPULSE_MULTIPLIER: f32 = -0.002;
//     const IMPULSE_MULTIPLIER: f32 = 0.5;
//     const ROT_DAMPING: f32 = 2.0;

//     for (mut thruster, transform, mut external_impulse, mut damping) in &mut query {
//         thruster.apply_clamping();

//         external_impulse.torque_impulse = thruster.rot * ROT_IMPULSE_MULTIPLIER;

//         let forward = transform.rotation * SHIP_MAIN_AXIS;
//         external_impulse.impulse = forward.xy() * (thruster.forward * IMPULSE_MULTIPLIER);
//         damping.angular_damping = if thruster.rot_damping {
//             ROT_DAMPING
//         } else {
//             0.0
//         };

//         *thruster = default();
//     }
// }

fn ship_thruster_system(
    mut query: Query<(
        &mut ShipThruster,
        &Transform,
        &mut ExternalForce,
        &mut Damping,
    )>,
) {
    const TORQUE_MULTIPLIER: f32 = -0.2;
    const FORWARD_FORCE_MULTIPLIER: f32 = 25.0;
    const ROT_DAMPING: f32 = 2.0;

    for (mut thruster, transform, mut external_force, mut damping) in &mut query {
        thruster.apply_clamping();

        external_force.torque = thruster.rot * TORQUE_MULTIPLIER;

        let forward = transform.rotation * SHIP_MAIN_AXIS;
        external_force.force = forward.xy() * (thruster.forward * FORWARD_FORCE_MULTIPLIER);
        damping.angular_damping = if thruster.rot_damping {
            ROT_DAMPING
        } else {
            0.0
        };

        // *thruster = default();
    }
}

fn ship_thruster_particle_system(
    mut query: Query<(&Parent, &mut ParticleSource), With<ShipThrusterParticleSpawner>>,
    ship_query: Query<(&ShipThruster, &Transform, &Velocity)>,
) {
    for (parent, mut particle_source) in &mut query {
        let Ok((thruster, transform, velocity)) = ship_query.get(parent.get()) else {
            continue;
        };
        // info!("thruster: {}", thruster.forward);
        if thruster.forward > 0.0 {
            let forward = transform.rotation * -SHIP_MAIN_AXIS;
            particle_source.rate = 50;
            particle_source.direction = ParticleDirection::DirectionalNormal {
                direction: -forward.xy().angle_between(Vec2::X),
                std_dev: 0.07,
            };
            particle_source.velocity_offset = velocity.linvel;
        } else {
            particle_source.rate = 0;
        }
    }
}
fn _ship_kinetic_debug_system(
    mut query: Query<(&Velocity, &ExternalImpulse, &ReadMassProperties)>,
) {
    for (velocity, external_impulse, mass) in &mut query {
        if external_impulse.torque_impulse != 0.0 {
            info!(
                "torque {} {} {}",
                external_impulse.torque_impulse,
                velocity.angvel,
                mass.get().principal_inertia
            );
        }
    }
}

fn _ship_attack_system_simple(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &AttackRequest, &mut WeaponState), With<ShipInput>>,
) {
    for (entity, transform, AttackRequest { primary_attack }, mut weapon_state) in query.iter_mut()
    {
        weapon_state.reload_timeout = (weapon_state.reload_timeout - time.delta_seconds()).max(0.0);
        if !primary_attack || weapon_state.reload_timeout > f32::EPSILON {
            continue;
        }
        weapon_state.reload_timeout = RELOAD_TIMEOUT;
        let direction = (transform.rotation * SHIP_MAIN_AXIS).xy();
        commands
            .spawn(weapon::KineticProjectileBundle::with_direction(
                entity, // *translation,
                direction,
            ))
            .insert(kinetic_projectile_shape_bundle(
                transform.translation,
                direction,
            ))
            .insert(Stroke::new(GREEN_HDR, 10.0));
    }
}
fn ship_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut query: Query<(Entity, &Transform, &AttackRequest, &mut WeaponState), With<ShipInput>>,
) {
    for (entity, transform, AttackRequest { primary_attack }, mut weapon_state) in query.iter_mut()
    {
        weapon_state.reload_timeout = (weapon_state.reload_timeout - time.delta_seconds()).max(0.0);
        if !primary_attack || weapon_state.reload_timeout > f32::EPSILON {
            continue;
        }

        let projectile_shape = Collider::ball(10.0);
        let projectile_pos = transform.translation.xy();
        let projectile_vel = (transform.rotation * SHIP_MAIN_AXIS).xy() * PROJECTILE_SPEED;
        let max_toi = 4.0;
        let filter = QueryFilter {
            exclude_collider: Some(entity),
            groups: Some(CollisionGroups {
                memberships: collision_groups::PROJECTILES,
                filters: collision_groups::DROIDS,
            }),

            ..default()
        };
        // QueryFilter {
        //     groups: Some(InteractionGroups {
        //         memberships: collision_groups::DROIDS.,
        //         filter: collision_groups::DROIDS,
        //     }),
        //     ..default()
        // };
        // if let Some((entity, hit)) = rapier_context.cast_shape(
        //     projectile_pos,
        //     default(),
        //     projectile_vel,
        //     &projectile_shape,
        //     max_toi,
        //     filter,
        // )
        {
            // The first collider hit has the entity `entity`. The `hit` is a
            // structure containing details about the hit configuration.
            // info!(
            //     "Hit the entity {:?} with the configuration: {:?}",
            //     entity, hit
            // );
            weapon_state.reload_timeout = RELOAD_TIMEOUT;
            let direction = (transform.rotation * SHIP_MAIN_AXIS).xy();
            commands
                .spawn(weapon::KineticProjectileBundle::with_direction(
                    entity, // *translation,
                    direction,
                ))
                .insert(kinetic_projectile_shape_bundle(
                    transform.translation,
                    direction,
                ))
                .insert(Stroke::new(GREEN_HDR, 10.0));
        }
    }
}

// fn cast_shape(rapier_context: Res<RapierContext>) {
//     let shape = Collider::cuboid(1.0, 2.0, 3.0);
//     let shape_pos = Vec3::new(1.0, 2.0, 3.0);
//     let shape_rot = Quat::from_rotation_z(0.8);
//     let shape_vel = Vec3::new(0.1, 0.4, 0.2);
//     let max_toi = 4.0;
//     let filter = QueryFilter::default();

//     if let Some((entity, hit)) =
//         rapier_context.cast_shape(shape_pos, shape_rot, shape_vel, &shape, max_toi, filter)
//     {
//         // The first collider hit has the entity `entity`. The `hit` is a
//         // structure containing details about the hit configuration.
//         println!(
//             "Hit the entity {:?} with the configuration: {:?}",
//             entity, hit
//         );
//     }
// }

pub struct ShipPlugin;
impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                apply_ship_input_system,
                ship_brake_maneuver_system.after(apply_ship_input_system),
                ship_thruster_system.after(ship_brake_maneuver_system),
                ship_attack_system,
                ship_thruster_particle_system.after(ship_thruster_system),
            ),
        )
        .add_systems(PostUpdate, ship_attach_thruster_particle_spawner_system);
    }
}
