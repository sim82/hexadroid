use bevy::prelude::*;

use big_brain::{
    evaluators::{Evaluator, LinearEvaluator},
    prelude::*,
};

use crate::droid::MovementStats;

use super::{EnemyEvaluation, IncomingProjectile, PredictedHit};

#[derive(Component, Debug, Clone, ScorerBuilder)]
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

#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct EnemyCloseScore;
pub fn enemy_close_system(
    mut scorer_query: Query<(&Actor, &mut Score), With<EnemyCloseScore>>,
    eval_query: Query<(&Parent, &EnemyEvaluation)>,
) {
    for (Actor(actor), mut score) in &mut scorer_query {
        if let Ok((_parent, eval)) = eval_query.get(*actor) {
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

#[derive(Component, Debug, Clone, ScorerBuilder)]
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

#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct IdleBoredomScore;
pub fn idle_boredom_score_system(
    mut scorer_query: Query<(&Actor, &mut Score), With<IdleBoredomScore>>,

    ai_query: Query<&Parent>,
    movement_stats_query: Query<&MovementStats>,
) {
    for (Actor(actor), mut score) in &mut scorer_query {
        let Ok(parent) = ai_query.get(*actor) else {
            continue;
        };
        let Ok(movement_stats) = movement_stats_query.get(parent.get()) else {
            continue;
        };
        score.set(
            LinearEvaluator::new_ranged(2.0, 7.0)
                .evaluate(movement_stats.idle_duration.as_secs_f32()),
        );
        // info!("idle boredom: {}", score.get());
    }
}
