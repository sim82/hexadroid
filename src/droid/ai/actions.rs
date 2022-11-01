use super::{EnemyEvaluation, IncomingProjectile, PredictedHit};
use crate::droid::{ai::DIRECTIONS, AttackRequest, TargetDirection, WeaponDirection};
use bevy::{prelude::*, utils::FloatOrd};
use big_brain::prelude::*;
use rand::{thread_rng, Rng};

#[derive(Component, Debug, Clone)]
pub struct ShootAction;

pub fn shoot_action_system(
    mut query: Query<(&Actor, &mut ActionState), With<ShootAction>>,
    mut attack_query: Query<(&mut AttackRequest, &mut WeaponDirection, &PredictedHit)>,
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
                if let Ok((mut attack_request, mut _weapon_direction)) =
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
                if let Ok(_target_direction) = direction_query.get_mut(*actor) {
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
