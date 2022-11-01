use self::{actions::ShootAction, scorers::EnemyHitScore};

use super::{weapon::Projectile, AttackRequest, TargetDirection, WeaponDirection, WeaponState};
use crate::{
    debug::DebugLinesExt, droid::weapon::PROJECTILE_SPEED, hex_point_to_vec2, CmdlineArgs,
    HEX_LAYOUT,
};
use bevy::{
    ecs::component,
    math::{Vec2Swizzles, Vec3Swizzles},
    prelude::*,
    time::FixedTimestep,
};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier2d::{prelude::*, rapier::prelude::Segment};
use big_brain::prelude::*;
use hexagon_tiles::{hexagon::HEX_DIRECTIONS, layout::LayoutTool};
use lazy_static::lazy_static;
use parry2d::{
    query::{self, NonlinearRigidMotion},
    shape::Ball,
};
use rand::prelude::*;

pub mod scorers {
    use bevy::{math::Vec3Swizzles, prelude::*};
    use bevy_prototype_debug_lines::DebugLines;
    use bevy_rapier2d::prelude::Velocity;
    use big_brain::{
        evaluators::{Evaluator, LinearEvaluator},
        prelude::*,
    };

    use crate::{debug::DebugLinesExt, droid::weapon::PROJECTILE_SPEED};

    use super::{
        EnemyEvaluation, IncomingProjectile, Line, PredictedHit, PrimaryEnemy, DIRECTIONS,
    };

    #[derive(Component, Debug, Clone)]
    pub struct EnemyHitScore;

    pub fn enemy_hit_score_system(
        mut scorer_query: Query<(&Actor, &mut Score), With<EnemyHitScore>>,
        predict_query: Query<&PredictedHit>,
    ) {
        for (Actor(actor), mut score) in &mut scorer_query {
            if let Ok(predicted) = predict_query.get(*actor) {
                // if projectile and enemy are predicted to reach intersection at roughly the
                // same time, shoot in this direction.
                let dt = predicted.dt;
                let value = LinearEvaluator::new_ranged(0.5, 0.0).evaluate(dt);
                // info!("hit: {}", value);
                score.set(value);
            }
        }
    }

    #[derive(Component, Debug, Clone)]
    pub struct EnemyCloseScore;
    pub fn enemy_close_system(
        mut scorer_query: Query<(&Actor, &mut Score), With<EnemyCloseScore>>,
        eval_query: Query<&EnemyEvaluation>,
    ) {
        for (Actor(actor), mut score) in &mut scorer_query {
            if let Ok(eval) = eval_query.get(*actor) {
                if eval.valid {
                    let value = LinearEvaluator::new_ranged(300.0, 100.0).evaluate(eval.distance);
                    debug!("score: {} {}", eval.distance, value);
                    score.set(value);
                } else {
                    score.set(0.0);
                }
            }
        }
    }

    #[derive(Component, Debug, Clone)]
    pub struct ProjectileIncomingScore;
    pub fn projectile_incoming_score_system(
        mut scorer_query: Query<(&Actor, &mut Score), With<ProjectileIncomingScore>>,
        eval_query: Query<&IncomingProjectile>,
    ) {
        for (Actor(actor), mut score) in &mut scorer_query {
            if let Ok(incoming) = eval_query.get(*actor) {
                let value = LinearEvaluator::new_ranged(0.5, 0.0).evaluate(incoming.toi);
                score.set(value);
            } else {
                score.set(0.0);
            }
        }
    }
}

pub mod actions {

    use std::f32::consts::E;

    use bevy::{math::Vec3Swizzles, prelude::*, utils::FloatOrd};
    use big_brain::prelude::*;
    use rand::{seq::SliceRandom, thread_rng, Rng};

    use crate::droid::{ai::DIRECTIONS, AttackRequest, TargetDirection, WeaponDirection};

    use super::{EnemyEvaluation, IncomingProjectile, PredictedHit, PrimaryEnemy};
    #[derive(Component, Debug, Clone)]
    pub struct ShootAction;

    pub fn shoot_action_system(
        mut query: Query<(&Actor, &mut ActionState), With<ShootAction>>,
        mut attack_query: Query<(&mut AttackRequest, &mut WeaponDirection, &PredictedHit)>,
        droid_query: Query<&Transform>,
    ) {
        for (Actor(actor), mut state) in &mut query {
            // info!("shoot action {:?}", state);
            match *state {
                ActionState::Requested => {
                    info!("shoot requested!");
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    info!("shoot!");

                    if let Ok((mut attack_request, mut weapon_direction, predicted)) =
                        attack_query.get_mut(*actor)
                    {
                        weapon_direction.direction = predicted.direction;
                        attack_request.primary_attack = true;
                    }
                    *state = ActionState::Success;
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    debug!("Action was cancelled. Considering this a failure.");
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }

    #[derive(Component, Debug, Clone)]
    pub struct IdleAction;
    pub fn idle_action_system(
        mut query: Query<(&Actor, &mut ActionState), With<IdleAction>>,
        mut attack_query: Query<(&mut AttackRequest, &mut WeaponDirection)>,
    ) {
        for (Actor(actor), mut state) in &mut query {
            // info!("idle action {:?}", state);
            match *state {
                ActionState::Requested => {
                    if let Ok((mut attack_request, mut weapon_direction)) =
                        attack_query.get_mut(*actor)
                    {
                        // weapon_direction.direction = DIRECTIONS[0];
                        attack_request.primary_attack = false;
                    }
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {

                    // *state = ActionState::Success;
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    debug!("Action was cancelled. Considering this a failure.");
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }

    #[derive(Component, Debug, Clone)]
    pub struct EvadeEnemyAction;

    pub fn evade_enemy_action_system(
        mut query: Query<(&Actor, &mut ActionState), With<EvadeEnemyAction>>,
        mut direction_query: Query<(&mut TargetDirection, &EnemyEvaluation, &mut AttackRequest)>,
    ) {
        for (Actor(actor), mut state) in &mut query {
            info!("evade action {:?}", state);
            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if let Ok((mut target_direction, enemy_eval, mut attack_request)) =
                        direction_query.get_mut(*actor)
                    {
                        attack_request.primary_attack = false;
                        if let Some(dir) = DIRECTIONS.iter().min_by_key(|dir| {
                            // if move_sideways {
                            //     1.0 - enemy_dir.dot(*dir).abs()
                            // } else {
                            FloatOrd(enemy_eval.direction.dot(**dir) + 1.0)
                            // }
                        }) {
                            target_direction.direction = *dir;
                        }
                    }
                    // *state = ActionState::Success;
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    if let Ok((mut target_direction, _enemy_eval, mut _attack_request)) =
                        direction_query.get_mut(*actor)
                    {
                        target_direction.direction = default();
                    }
                    debug!("Action was cancelled. Considering this a failure.");
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }

    #[derive(Component, Debug, Clone, Default)]
    pub struct EvadeProjectileAction {
        direction: Vec2,
    }

    pub fn evade_projectile_action_system(
        mut query: Query<(&Actor, &mut ActionState, &mut EvadeProjectileAction)>,
        mut direction_query: Query<&mut TargetDirection>,
        incoming_query: Query<&IncomingProjectile>,
    ) {
        for (Actor(actor), mut state, mut evade) in &mut query {
            info!("evade projectile action {:?}", state);
            match *state {
                ActionState::Requested => {
                    if let Ok(mut target_direction) = direction_query.get_mut(*actor) {
                        if let Ok(incoming_projectile) = incoming_query.get(*actor) {
                            let mut rng = thread_rng();
                            if let Some(dir) = DIRECTIONS.iter().min_by_key(|dir| {
                                // if move_sideways {
                                //     1.0 - enemy_dir.dot(*dir).abs()
                                // } else {
                                FloatOrd(
                                    incoming_projectile.velocity.normalize().dot(**dir).abs()
                                        + rng.gen_range(-0.1..0.1),
                                )
                                // }
                            }) {
                                evade.direction = *dir;
                            }
                        } else {
                            *state = ActionState::Cancelled;
                            continue;
                        }
                    }

                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if let Ok(mut target_direction) = direction_query.get_mut(*actor) {
                        if incoming_query.contains(*actor) {
                            target_direction.direction = evade.direction;
                        } else {
                            target_direction.direction = default();
                            *state = ActionState::Success;
                        }
                    }
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    if let Ok(mut target_direction) = direction_query.get_mut(*actor) {
                        target_direction.direction = default();
                    }
                    debug!("Action was cancelled. Considering this a failure.");
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

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

#[derive(Component, Default)]
pub struct MovementState {
    time_left: f32,
    direction: Vec2,
}

#[derive(Copy, Clone, Debug)]
struct Line(Vec2, Vec2);

impl Line {
    #[allow(unused)]
    // explicit line-line intersection. Keep for reference
    pub fn intersect_explicit(self, other: Self) -> Option<Vec2> {
        // pub fn lines_intersect_2d( p0 : Vec2, p1: Vec2, Vector2 const& p2, Vector2 const& p3, Vector2* i const = 0) {
        let Line(p0, p1) = self;
        let Line(p2, p3) = other;
        let s1 = p1 - p0;
        let s2 = p3 - p2;

        let u = p0 - p2;

        let ip = 1f32 / (-s2.x * s1.y + s1.x * s2.y);

        let s = (-s1.y * u.x + s1.x * u.y) * ip;
        let t = (s2.x * u.y - s2.y * u.x) * ip;

        if (0.0..=1.0).contains(&s) && (0.0..=1.0).contains(&t) {
            Some(p0 + (s1 * t))
        } else {
            None
        }
    }

    pub fn intersect(self, other: Self) -> Option<Vec2> {
        let self_seg = Segment::new(self.0.into(), self.1.into());
        let other_seg = Segment::new(other.0.into(), other.1.into());
        if let Ok(Some(contact)) =
            query::contact(&default(), &self_seg, &default(), &other_seg, 0.0)
        {
            Some(contact.point1.into())
        } else {
            None
        }
    }
}

#[derive(Component, Default)]
pub struct EnemyEvaluation {
    valid: bool,
    distance: f32,
    direction: Vec2,
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
    pos: Vec2,
    velocity: Vec2,
    toi: f32,
}

fn incomping_projectile_evaluation_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<PrimaryEnemy>>,
    projectile_query: Query<(&Transform, &Velocity), With<Projectile>>,
) {
    for (entity, my_transform) in &query {
        let my_pos = my_transform.translation.xy().into();
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
                0.5,
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

fn assault_predict_system_explicit(
    enemy_query: Query<(&Transform, &Velocity)>,
    mut debug_lines: Option<ResMut<DebugLines>>,
    mut assault_query: Query<(&Transform, &PrimaryEnemy, &mut PredictedHit, &WeaponState)>,
) {
    for (
        Transform {
            translation: my_translation,
            ..
        },
        PrimaryEnemy { enemy },
        mut predicted_hit,
        weapon_state,
    ) in assault_query.iter_mut()
    {
        if let Ok((
            Transform {
                translation: enemy_translation,
                ..
            },
            Velocity {
                linvel: enemy_velocity,
                ..
            },
        )) = enemy_query.get(*enemy)
        {
            let enemy_speed = enemy_velocity.length();

            if enemy_speed <= f32::EPSILON || weapon_state.reload_timeout > f32::EPSILON {
                // enemy not moving
                predicted_hit.dt = f32::INFINITY;
                continue;
            }

            let mut best_dt = f32::INFINITY;

            for dir in DIRECTIONS.iter() {
                // find intersection between predicted projectile and enemy trajectories
                let enemy_line = Line(
                    enemy_translation.xy(),
                    enemy_translation.xy() + *enemy_velocity,
                );
                let projectile_start_pos = my_translation.xy() + *dir * 50.0;
                let projectile_line = Line(
                    projectile_start_pos,
                    my_translation.xy() + *dir * PROJECTILE_SPEED,
                );

                if let Some(debug_lines) = debug_lines.as_mut() {
                    debug_lines.line(enemy_line.0.extend(0.0), enemy_line.1.extend(0.0), 0.0);
                    debug_lines.line(
                        projectile_line.0.extend(0.0),
                        projectile_line.1.extend(0.0),
                        0.0,
                    );
                }
                if let Some(intersect) = projectile_line.intersect(enemy_line) {
                    // predicted 'time to intersection'
                    let my_d = (intersect - projectile_start_pos).length();
                    let enemy_d = (intersect - enemy_translation.xy()).length();

                    let my_t = my_d / PROJECTILE_SPEED;
                    let enemy_t = enemy_d / enemy_speed;

                    let dt = (my_t - enemy_t).abs();

                    if let Some(debug_lines) = debug_lines.as_mut() {
                        info!("t: {} {}", my_t, enemy_t);
                        let duration = if dt < 0.1 { 1.0 } else { 0.0 };
                        debug_lines.cross(
                            (projectile_start_pos
                                + (intersect - projectile_start_pos) / my_t * enemy_t)
                                .extend(0.0),
                            duration,
                        );
                    }

                    if dt < best_dt {
                        predicted_hit.direction = *dir;
                        predicted_hit.dt = dt;
                        best_dt = dt;
                    }
                }
            }
        }
    }
}

fn assault_predict_system(
    enemy_query: Query<(&Transform, &Velocity)>,
    mut debug_lines: Option<ResMut<DebugLines>>,
    mut assault_query: Query<(&Transform, &PrimaryEnemy, &mut PredictedHit, &WeaponState)>,
) {
    for (
        Transform {
            translation: my_translation,
            ..
        },
        PrimaryEnemy { enemy },
        mut predicted_hit,
        weapon_state,
    ) in assault_query.iter_mut()
    {
        if let Ok((
            Transform {
                translation: enemy_translation,
                ..
            },
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
            let enemy_start_pos = enemy_translation.xy().into();
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
                    1.0,
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

fn movement_update_system(
    time: Res<Time>,
    mut query: Query<(
        &PrimaryEnemy,
        &mut TargetDirection,
        &Transform,
        &mut MovementState,
    )>,
    enemy_query: Query<&Transform>,
    mut debug_lines: Option<ResMut<DebugLines>>,
    args: Res<CmdlineArgs>,
) {
    if args.gravity {
        return;
    }
    for (
        PrimaryEnemy { enemy },
        mut target_direction,
        Transform {
            translation: my_pos,
            ..
        },
        mut movement_state,
    ) in query.iter_mut()
    {
        movement_state.time_left -= time.delta_seconds();
        if movement_state.time_left > 0.0 {
            continue;
        }
        movement_state.time_left = 1.0;

        if let Ok(Transform {
            translation: enemy_pos,
            ..
        }) = enemy_query.get(*enemy)
        {
            let enemy_dir = (*enemy_pos - *my_pos).xy().normalize_or_zero();
            let mut rng = rand::thread_rng();
            let move_sideways = rng.gen_bool(0.5);

            info!("move sideways: {:?}", move_sideways);

            if let Ok(dir) = DIRECTIONS.choose_weighted(&mut rng, {
                |dir| {
                    if move_sideways {
                        1.0 - enemy_dir.dot(*dir).abs()
                    } else {
                        enemy_dir.dot(*dir) + 1.0
                    }
                }
            }) {
                target_direction.direction = *dir;

                if let Some(debug_lines) = &mut debug_lines {
                    let color = if move_sideways {
                        Color::BLUE
                    } else {
                        Color::RED
                    };
                    debug_lines.line_colored(
                        *my_pos,
                        *my_pos + (dir.extend(0.0) * 16.0),
                        1.0,
                        color,
                    );
                }
            }
        }
    }
}

const LABEL: &str = "my_fixed_timestep";
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

pub struct AiPlugin;
impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BigBrainPlugin);

        app.add_stage_after(
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.1).with_label(LABEL))
                .with_system(assault_predict_system)
                .with_system(incomping_projectile_evaluation_system),
        );

        app.add_system(movement_update_system)
            .add_system(enemy_evaluation_system)
            .add_system_to_stage(BigBrainStage::Actions, actions::shoot_action_system)
            .add_system_to_stage(BigBrainStage::Actions, actions::idle_action_system)
            .add_system_to_stage(BigBrainStage::Actions, actions::evade_enemy_action_system)
            .add_system_to_stage(
                BigBrainStage::Actions,
                actions::evade_projectile_action_system,
            )
            .add_system_to_stage(BigBrainStage::Scorers, scorers::enemy_hit_score_system)
            .add_system_to_stage(
                BigBrainStage::Scorers,
                scorers::projectile_incoming_score_system,
            )
            .add_system_to_stage(BigBrainStage::Scorers, scorers::enemy_close_system);
    }
}

#[derive(Bundle, Default)]
pub struct AssaultAiBundle {
    assault_ai: AssaultAi,
    movement_state: MovementState,
}

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
        .when(FixedScore(0.1), actions::IdleAction)
}
