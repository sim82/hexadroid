use self::{actions::ShootAction, scorers::EnemyHitScore};

use super::{AttackRequest, TargetDirection, WeaponDirection, WeaponState};
use crate::{
    debug::DebugLinesExt, droid::weapon::PROJECTILE_SPEED, hex_point_to_vec2, CmdlineArgs,
    HEX_LAYOUT,
};
use bevy::{ecs::component, math::Vec3Swizzles, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier2d::prelude::*;
use big_brain::prelude::*;
use hexagon_tiles::{hexagon::HEX_DIRECTIONS, layout::LayoutTool};
use lazy_static::lazy_static;
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

    use super::{Line, PredictedHit, PrimaryEnemy, DIRECTIONS};

    #[derive(Component, Debug, Clone)]
    pub struct EnemyHitScore;

    // pub fn enemy_hit_score_system(
    //     mut debug_lines: Option<ResMut<DebugLines>>,
    //     mut scorer_query: Query<(&Actor, &mut Score), With<EnemyHitScore>>,
    //     enemy_query: Query<&PrimaryEnemy>,
    //     droid_query: Query<(&Transform, &Velocity)>,
    // ) {
    //     for (Actor(actor), mut score) in &mut scorer_query {
    //         if let Ok(PrimaryEnemy { enemy }) = enemy_query.get(*actor) {
    //             if let (Ok((my_transform, _my_velocity)), Ok((enemy_transform, enemy_velocity))) =
    //                 (droid_query.get(*actor), droid_query.get(*enemy))
    //             {
    //                 let _enemy_dir = enemy_velocity.linvel.normalize_or_zero();
    //                 let enemy_speed = enemy_velocity.linvel.length();

    //                 if enemy_speed <= f32::EPSILON {
    //                     // enemy not moving
    //                     score.set(0.0);
    //                     continue;
    //                 }

    //                 for dir in DIRECTIONS.iter() {
    //                     // find intersection between predicted projectile and enemy trajectories
    //                     let enemy_line = Line(
    //                         enemy_transform.translation.xy(),
    //                         enemy_transform.translation.xy() + enemy_velocity.linvel,
    //                     );
    //                     let projectile_start_pos = my_transform.translation.xy() + *dir * 50.0;
    //                     let projectile_line = Line(
    //                         projectile_start_pos,
    //                         my_transform.translation.xy() + *dir * PROJECTILE_SPEED,
    //                     );

    //                     if let Some(debug_lines) = debug_lines.as_mut() {
    //                         debug_lines.line(
    //                             enemy_line.0.extend(0.0),
    //                             enemy_line.1.extend(0.0),
    //                             0.0,
    //                         );
    //                         debug_lines.line(
    //                             projectile_line.0.extend(0.0),
    //                             projectile_line.1.extend(0.0),
    //                             0.0,
    //                         );
    //                     }
    //                     if let Some(intersect) = projectile_line.intersect2(enemy_line) {
    //                         // predicted 'time to intersection'
    //                         let my_d = (intersect - projectile_start_pos).length();
    //                         let enemy_d = (intersect - enemy_transform.translation.xy()).length();

    //                         let my_t = my_d / PROJECTILE_SPEED;
    //                         let enemy_t = enemy_d / enemy_speed;

    //                         if let Some(debug_lines) = debug_lines.as_mut() {
    //                             debug_lines.cross(
    //                                 (projectile_start_pos
    //                                     + (intersect - projectile_start_pos) / my_t * enemy_t)
    //                                     .extend(0.0),
    //                                 0.0,
    //                             );
    //                         }
    //                         // if projectile and enemy are predicted to reach intersection at roughly the
    //                         // same time, shoot in this direction.
    //                         let dt = (my_t - enemy_t).abs();
    //                         let value = LinearEvaluator::new_ranged(0.5, 0.0).evaluate(dt);
    //                         // info!("hit: {}", value);
    //                         score.set(value);
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

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
}

pub mod actions {

    use bevy::{math::Vec3Swizzles, prelude::*, utils::FloatOrd};
    use big_brain::prelude::*;

    use crate::droid::{ai::DIRECTIONS, AttackRequest, WeaponDirection};

    use super::{PredictedHit, PrimaryEnemy};
    #[derive(Component, Debug, Clone)]
    pub struct ShootAction;

    // pub fn shoot_action_system(
    //     mut query: Query<(&Actor, &mut ActionState), With<ShootAction>>,
    //     mut attack_query: Query<(
    //         &mut AttackRequest,
    //         &mut WeaponDirection,
    //         &PrimaryEnemy,
    //         &Transform,
    //     )>,
    //     droid_query: Query<&Transform>,
    // ) {
    //     for (Actor(actor), mut state) in &mut query {
    //         // info!("shoot action {:?}", state);
    //         match *state {
    //             ActionState::Requested => {
    //                 info!("shoot requested!");
    //                 *state = ActionState::Executing;
    //             }
    //             ActionState::Executing => {
    //                 info!("shoot!");

    //                 if let Ok((
    //                     mut attack_request,
    //                     mut weapon_direction,
    //                     PrimaryEnemy { enemy },
    //                     my_transform,
    //                 )) = attack_query.get_mut(*actor)
    //                 {
    //                     if let Ok(enemy_transform) = droid_query.get(*enemy) {
    //                         let enemy_dir =
    //                             (enemy_transform.translation - my_transform.translation).xy();
    //                         weapon_direction.direction = *DIRECTIONS
    //                             .iter()
    //                             .max_by_key(|aim_dir| FloatOrd(aim_dir.dot(enemy_dir)))
    //                             .unwrap_or(&DIRECTIONS[0]);
    //                     }

    //                     attack_request.primary_attack = true;
    //                 }
    //                 *state = ActionState::Success;
    //             }
    //             // All Actions should make sure to handle cancellations!
    //             ActionState::Cancelled => {
    //                 debug!("Action was cancelled. Considering this a failure.");
    //                 *state = ActionState::Failure;
    //             }
    //             _ => {}
    //         }
    //     }
    // }

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
    pub fn intersect(self, other: Self) -> Option<Vec2> {
        info!("intersect: {:?} {:?}", self, other);
        let a1 = self.1.y - self.0.y;
        let b1 = self.0.x - self.1.x;
        let c1 = a1 * self.0.x + b1 * self.0.y;

        let a2 = other.1.y - other.0.y;
        let b2 = other.0.x - other.1.x;
        let c2 = a2 * other.0.x + b2 * other.0.y;

        let delta = a1 * b2 - a2 * b1;

        if delta.abs() < f32::EPSILON {
            return None;
        }

        Some(Vec2::new(
            (b2 * c1 - b1 * c2) / delta,
            (a1 * c2 - a2 * c1) / delta,
        ))
    }

    pub fn intersect2(self, other: Self) -> Option<Vec2> {
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

        // }
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
    enemy_query: Query<(&Transform, &Velocity)>,
    mut debug_lines: Option<ResMut<DebugLines>>,
    mut assault_query: Query<(&Transform, &PrimaryEnemy, &mut PredictedHit)>,
) {
    for (
        Transform {
            translation: my_translation,
            ..
        },
        PrimaryEnemy { enemy },
        mut predicted_hit,
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

            if enemy_speed <= f32::EPSILON {
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
                if let Some(intersect) = projectile_line.intersect2(enemy_line) {
                    // predicted 'time to intersection'
                    let my_d = (intersect - projectile_start_pos).length();
                    let enemy_d = (intersect - enemy_translation.xy()).length();

                    let my_t = my_d / PROJECTILE_SPEED;
                    let enemy_t = enemy_d / enemy_speed;

                    if let Some(debug_lines) = debug_lines.as_mut() {
                        debug_lines.cross(
                            (projectile_start_pos
                                + (intersect - projectile_start_pos) / my_t * enemy_t)
                                .extend(0.0),
                            0.0,
                        );
                    }
                    let dt = (my_t - enemy_t).abs();

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

pub struct AiPlugin;
impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BigBrainPlugin);

        app.add_system(assault_predict_system)
            .add_system(movement_update_system)
            .add_system_to_stage(BigBrainStage::Actions, actions::shoot_action_system)
            .add_system_to_stage(BigBrainStage::Actions, actions::idle_action_system)
            .add_system_to_stage(BigBrainStage::Scorers, scorers::enemy_hit_score_system);
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
        .picker(FirstToScore { threshold: 0.8 })
        .when(scorers::EnemyHitScore, actions::ShootAction)
        .otherwise(actions::IdleAction)
}
