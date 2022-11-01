use bevy::prelude::*;

use big_brain::{
    evaluators::{Evaluator, LinearEvaluator},
    prelude::*,
};

use super::{EnemyEvaluation, IncomingProjectile, PredictedHit};

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
