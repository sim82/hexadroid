use crate::player::PlayerMarker;
use crate::prelude::*;
use crate::weapon::PROJECTILE_SPEED;
use crate::{droid::WeaponState, weapon::Projectile};
use bevy::{
    math::{Vec2Swizzles, Vec3Swizzles},
    prelude::*,
};
use bevy_rapier2d::parry::{
    query::{self},
    shape::Ball,
};
use bevy_rapier2d::prelude::*;
use big_brain::prelude::*;
use lazy_static::lazy_static;

pub mod actions;
pub mod scorers;

const SQRT2_2: f32 = std::f32::consts::SQRT_2 / 2.0;
lazy_static! {
    static ref DIRECTIONS: [Vec2; 8] = [
        Vec2::new(1.0, 0.0),
        Vec2::new(SQRT2_2, -SQRT2_2),
        Vec2::new(0.0, -1.0),
        Vec2::new(-SQRT2_2, -SQRT2_2),
        Vec2::new(-1.0, 0.0),
        Vec2::new(-SQRT2_2, SQRT2_2),
        Vec2::new(0.0, 1.0),
        Vec2::new(SQRT2_2, SQRT2_2),
    ];
}

#[derive(Component, Default)]
pub struct AssaultAi {}
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct PrimaryEnemy {
    pub enemy: Entity,
}

// #[derive(Component, Default)]
// pub struct MovementState {
//     time_left: f32,
//     direction: Vec2,
// }

// #[derive(Copy, Clone, Debug)]
// struct Line(Vec2, Vec2);

// impl Line {
//     #[allow(unused)]
//     // explicit line-line intersection. Keep for reference
//     pub fn intersect_explicit(self, other: Self) -> Option<Vec2> {
//         // pub fn lines_intersect_2d( p0 : Vec2, p1: Vec2, Vector2 const& p2, Vector2 const& p3, Vector2* i const = 0) {
//         let Line(p0, p1) = self;
//         let Line(p2, p3) = other;
//         let s1 = p1 - p0;
//         let s2 = p3 - p2;

//         let u = p0 - p2;

//         let ip = 1f32 / (-s2.x * s1.y + s1.x * s2.y);

//         let s = (-s1.y * u.x + s1.x * u.y) * ip;
//         let t = (s2.x * u.y - s2.y * u.x) * ip;

//         if (0.0..=1.0).contains(&s) && (0.0..=1.0).contains(&t) {
//             Some(p0 + (s1 * t))
//         } else {
//             None
//         }
//     }

//     pub fn intersect(self, other: Self) -> Option<Vec2> {
//         let self_seg = Segment::new(self.0.into(), self.1.into());
//         let other_seg = Segment::new(other.0.into(), other.1.into());
//         if let Ok(Some(contact)) =
//             query::contact(&default(), &self_seg, &default(), &other_seg, 0.0)
//         {
//             Some(contact.point1.into())
//         } else {
//             None
//         }
//     }
// }

#[derive(Component, Default)]
pub struct EnemyEvaluation {
    valid: bool,
    distance: f32,
    direction: Vec2,
}

fn enemy_select_system(
    mut query: Query<&mut PrimaryEnemy>,

    player_query: Query<Entity, With<PlayerMarker>>,
) {
    let Some(enemy) = player_query.iter().next() else {
        return;
    };
    for mut primary_enemy in &mut query {
        primary_enemy.enemy = enemy;
    }
}
fn enemy_evaluation_system(
    mut query: Query<(&mut EnemyEvaluation, &PrimaryEnemy, &Transform)>,
    droid_query: Query<&Transform>,
) {
    for (mut eval, enemy, my_transform) in &mut query {
        if let Ok(enemy_transform) = droid_query.get(enemy.enemy) {
            eval.direction = (enemy_transform.translation - my_transform.translation).xy();
            eval.distance = eval.direction.length();
            eval.valid = true;
        } else {
            eval.valid = false;
        }
    }
}

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct IncomingProjectile {
    pub pos: Vec2,
    pub velocity: Vec2,
    pub toi: f32,
}

fn incoming_projectile_evaluation_system(
    mut commands: Commands,
    query: Query<(Entity, &GlobalTransform), With<PrimaryEnemy>>,
    projectile_query: Query<(&Transform, &Velocity), With<Projectile>>,
) {
    for (entity, my_transform) in &query {
        let my_pos = my_transform.translation().xy().into();
        let my_vel = default();
        let my_shape = Ball::new(28.0);
        let projectile_shape = Ball::new(10.0);
        let mut lowest_toi = f32::INFINITY;

        let mut incoming = None;

        for (projectile_transform, projectile_velocity) in &projectile_query {
            let projectile_pos = projectile_transform.translation.xy().into();
            let projectile_vel = projectile_velocity.linvel.xy().into();
            if let Ok(Some(toi)) = query::time_of_impact(
                &my_pos,
                &my_vel,
                &my_shape,
                &projectile_pos,
                &projectile_vel,
                &projectile_shape,
                0.3,
                true,
            ) {
                if toi.toi < lowest_toi {
                    incoming = Some(IncomingProjectile {
                        pos: projectile_transform.translation.xy(),
                        velocity: projectile_vel.into(),
                        toi: toi.toi,
                    });
                    lowest_toi = toi.toi;
                }
            }
        }
        if let Some(incoming) = incoming {
            commands.entity(entity).insert(incoming);
            // info!("incoming");
        } else {
            commands.entity(entity).remove::<IncomingProjectile>();
            // info!("remove incoming");
        }
    }
}

#[derive(Component)]
pub struct PredictedHit {
    direction: Vec2,
    dt: f32,
}

impl Default for PredictedHit {
    fn default() -> Self {
        Self {
            direction: Default::default(),
            dt: f32::INFINITY,
        }
    }
}

fn assault_predict_system(
    enemy_query: Query<(&GlobalTransform, &Velocity)>,
    // mut debug_lines: Option<ResMut<DebugLines>>,
    mut assault_query: Query<(&Parent, &GlobalTransform, &PrimaryEnemy, &mut PredictedHit)>,
    droid_query: Query<&WeaponState>,
) {
    for (
        parent,
        global_transform,
        PrimaryEnemy { enemy },
        mut predicted_hit,
        // weapon_state,
    ) in assault_query.iter_mut()
    {
        let my_translation = global_transform.translation();
        let Ok(weapon_state) = droid_query.get(parent.get()) else {
            continue;
        };
        if let Ok((
            enemy_transform,
            Velocity {
                linvel: enemy_velocity,
                ..
            },
        )) = enemy_query.get(*enemy)
        {
            if weapon_state.reload_timeout > f32::EPSILON {
                // enemy not moving
                predicted_hit.dt = f32::INFINITY;
                continue;
            }

            let mut lowest_toi = f32::INFINITY;
            let enemy_shape = Ball::new(28.0);
            let enemy_start_pos = enemy_transform.translation().xy().into();
            let enemy_vel = enemy_velocity.xy().into();

            let projectile_shape = Ball::new(10.0);
            let projectile_start_pos = my_translation.xy().into();

            for dir in DIRECTIONS.iter() {
                let projectile_vel = (*dir * PROJECTILE_SPEED).into();
                if let Ok(Some(toi)) = query::time_of_impact(
                    &projectile_start_pos,
                    &projectile_vel,
                    &projectile_shape,
                    &enemy_start_pos,
                    &enemy_vel,
                    &enemy_shape,
                    0.7,
                    true,
                ) {
                    if toi.toi < lowest_toi {
                        predicted_hit.dt = 0.0;
                        predicted_hit.direction = *dir;
                        lowest_toi = toi.toi;
                    }
                }
            }
        }
    }
}

// fn movement_update_system(
//     time: Res<Time>,
//     mut query: Query<(
//         &PrimaryEnemy,
//         &mut TargetDirection,
//         &Transform,
//         &mut MovementState,
//     )>,
//     enemy_query: Query<&Transform>,
//     mut debug_lines: Option<ResMut<DebugLines>>,
//     args: Res<CmdlineArgs>,
// ) {
//     if args.gravity {
//         return;
//     }
//     for (
//         PrimaryEnemy { enemy },
//         mut target_direction,
//         Transform {
//             translation: my_pos,
//             ..
//         },
//         mut movement_state,
//     ) in query.iter_mut()
//     {
//         movement_state.time_left -= time.delta_seconds();
//         if movement_state.time_left > 0.0 {
//             continue;
//         }
//         movement_state.time_left = 1.0;

//         if let Ok(Transform {
//             translation: enemy_pos,
//             ..
//         }) = enemy_query.get(*enemy)
//         {
//             let enemy_dir = (*enemy_pos - *my_pos).xy().normalize_or_zero();
//             let mut rng = rand::thread_rng();
//             let move_sideways = rng.gen_bool(0.5);

//             info!("move sideways: {:?}", move_sideways);

//             if let Ok(dir) = DIRECTIONS.choose_weighted(&mut rng, {
//                 |dir| {
//                     if move_sideways {
//                         1.0 - enemy_dir.dot(*dir).abs()
//                     } else {
//                         enemy_dir.dot(*dir) + 1.0
//                     }
//                 }
//             }) {
//                 target_direction.direction = *dir;

//                 if let Some(debug_lines) = &mut debug_lines {
//                     let color = if move_sideways {
//                         Color::BLUE
//                     } else {
//                         Color::RED
//                     };
//                     debug_lines.line_colored(
//                         *my_pos,
//                         *my_pos + (dir.extend(0.0) * 16.0),
//                         1.0,
//                         color,
//                     );
//                 }
//             }
//         }
//     }
// }

// const LABEL: &str = "my_fixed_timestep";
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct FixedUpdateStage;

pub struct AiPlugin;
impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigBrainPlugin::new(PreUpdate));

        // app.add_stage_after(
        //     CoreStage::Update,
        //     FixedUpdateStage,
        //     SystemStage::parallel()
        //         .with_run_criteria(FixedTimestep::step(0.1).with_label(LABEL))
        //         .with_system(assault_predict_system)
        //         .with_system(incomping_projectile_evaluation_system),
        // );

        app.add_systems(PreUpdate, enemy_select_system);
        app.add_systems(
            Update,
            (
                assault_predict_system,
                incoming_projectile_evaluation_system,
            )
                .run_if(in_state(GameState::Game)),
        );
        // app
        //     // .add_system(movement_update_system)
        //     .add_system(enemy_evaluation_system)
        //     .add_system_to_stage(BigBrainStage::Actions, actions::shoot_action_system)
        //     .add_system_to_stage(BigBrainStage::Actions, actions::idle_action_system)
        //     .add_system_to_stage(BigBrainStage::Actions, actions::evade_enemy_action_system)
        //     .add_system_to_stage(
        //         BigBrainStage::Actions,
        //         actions::evade_projectile_action_system,
        //     )
        //     .add_system_to_stage(BigBrainStage::Scorers, scorers::enemy_hit_score_system)
        //     .add_system_to_stage(
        //         BigBrainStage::Scorers,
        //         scorers::projectile_incoming_score_system,
        //     )
        //     .add_system_to_stage(BigBrainStage::Scorers, scorers::enemy_close_system);
        app.add_systems(Update, enemy_evaluation_system);
        app.add_systems(
            PreUpdate,
            (
                actions::shoot_action_system.in_set(BigBrainSet::Actions),
                actions::idle_action_system.in_set(BigBrainSet::Actions),
                actions::evade_enemy_action_system.in_set(BigBrainSet::Actions),
                actions::evade_projectile_action_system.in_set(BigBrainSet::Actions),
                actions::roam_action_system.in_set(BigBrainSet::Actions),
            )
                .run_if(in_state(GameState::Game)),
        );
        app.add_systems(
            PreUpdate,
            (
                scorers::enemy_hit_score_system.in_set(BigBrainSet::Scorers),
                scorers::projectile_incoming_score_system.in_set(BigBrainSet::Scorers),
                scorers::enemy_close_system.in_set(BigBrainSet::Scorers),
                scorers::idle_boredom_score_system.in_set(BigBrainSet::Scorers),
            )
                .run_if(in_state(GameState::Game)),
        );
    }
}

// #[derive(Bundle, Default)]
// pub struct AssaultAiBundle {
//     assault_ai: AssaultAi,
//     movement_state: MovementState,
// }

// impl AssaultAiBundle {
//     pub fn new() -> Self {
//         Self {
//             assault_ai: AssaultAi {},
//         }
//     }
// }

pub fn new_shooting_droid_ai() -> ThinkerBuilder {
    Thinker::build()
        .label("shooting droid")
        .picker(Highest)
        .when(scorers::EnemyHitScore, actions::ShootAction)
        .when(
            scorers::ProjectileIncomingScore,
            actions::EvadeProjectileAction::default(),
        )
        .when(scorers::EnemyCloseScore, actions::EvadeEnemyAction)
        .when(scorers::IdleBoredomScore, actions::RoamAction::default())
        .when(FixedScore::build(0.1), actions::IdleAction)
}
