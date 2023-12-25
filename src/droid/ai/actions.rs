use super::{EnemyEvaluation, IncomingProjectile, PredictedHit};
use crate::droid::{ai::DIRECTIONS, AttackRequest, TargetDirection, WeaponDirection};
use bevy::{prelude::*, utils::FloatOrd};
use big_brain::prelude::*;
use rand::{thread_rng, Rng};

#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct ShootAction;

pub fn shoot_action_system(
    mut query: Query<(&Actor, &mut ActionState), With<ShootAction>>,
    ai_query: Query<(&PredictedHit, &Parent)>,
    mut attack_query: Query<(&mut AttackRequest, &mut WeaponDirection)>,
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

                let Ok((predicted, parent)) = ai_query.get(*actor) else {
                    continue;
                };
                if let Ok((mut attack_request, mut weapon_direction)) =
                    attack_query.get_mut(parent.get())
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

#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct IdleAction;
pub fn idle_action_system(
    mut query: Query<(&Actor, &mut ActionState), With<IdleAction>>,
    ai_query: Query<&Parent>,
    mut attack_query: Query<(&mut AttackRequest, &mut WeaponDirection)>,
) {
    for (Actor(actor), mut state) in &mut query {
        // info!("idle action {:?}", state);
        match *state {
            ActionState::Requested => {
                let Ok(parent) = ai_query.get(*actor) else {
                    continue;
                };
                if let Ok((mut attack_request, mut _weapon_direction)) =
                    attack_query.get_mut(parent.get())
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

#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct EvadeEnemyAction;

pub fn evade_enemy_action_system(
    mut query: Query<(&Actor, &mut ActionState), With<EvadeEnemyAction>>,
    ai_query: Query<(&Parent, &EnemyEvaluation)>,
    mut direction_query: Query<(&mut TargetDirection, &mut AttackRequest)>,
) {
    for (Actor(actor), mut state) in &mut query {
        debug!("evade action {:?}", state);
        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                let Ok((parent, enemy_eval)) = ai_query.get(*actor) else {
                    continue;
                };
                if let Ok((mut target_direction, mut attack_request)) =
                    direction_query.get_mut(parent.get())
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
                let Ok((parent, _)) = ai_query.get(*actor) else {
                    continue;
                };
                if let Ok((mut target_direction, mut _attack_request)) =
                    direction_query.get_mut(parent.get())
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

#[derive(Component, Debug, Clone, Default, ActionBuilder)]
pub struct EvadeProjectileAction {
    direction: Vec2,
}

pub fn evade_projectile_action_system(
    mut query: Query<(&Actor, &mut ActionState, &mut EvadeProjectileAction)>,
    ai_query: Query<&Parent>,
    mut direction_query: Query<&mut TargetDirection>,
    incoming_query: Query<&IncomingProjectile>,
) {
    for (Actor(actor), mut state, mut evade) in &mut query {
        info!("evade projectile action {:?}", state);
        match *state {
            ActionState::Requested => {
                let Ok(parent) = ai_query.get(*actor) else {
                    continue;
                };
                if let Ok(_target_direction) = direction_query.get_mut(parent.get()) {
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
                let Ok(parent) = ai_query.get(*actor) else {
                    *state = ActionState::Success;
                    continue;
                };
                if let Ok(mut target_direction) = direction_query.get_mut(parent.get()) {
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
                let Ok(parent) = ai_query.get(*actor) else {
                    continue;
                };
                if let Ok(mut target_direction) = direction_query.get_mut(parent.get()) {
                    target_direction.direction = default();
                }
                debug!("Action was cancelled. Considering this a failure.");
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

#[derive(Component, Debug, Clone, Default, ActionBuilder)]
pub struct RoamAction {
    direction: Vec2,
    timer: Timer,
}

pub fn roam_action_system(
    time: Res<Time>,
    mut query: Query<(&Actor, &mut ActionState, &mut RoamAction)>,
    ai_query: Query<&Parent>,
    mut direction_query: Query<&mut TargetDirection>,
) {
    for (Actor(actor), mut state, mut roam) in &mut query {
        debug!("roam action {:?}", state);
        match *state {
            ActionState::Requested => {
                let mut rng = thread_rng();

                roam.direction = DIRECTIONS[rng.gen_range(0..6)];
                roam.timer = Timer::from_seconds(0.5, TimerMode::Once);
                *state = ActionState::Executing;
            }
            ActionState::Executing | ActionState::Cancelled => {
                roam.timer.tick(time.delta());
                let Ok(parent) = ai_query.get(*actor) else {
                    continue;
                };
                if let Ok(mut target_direction) = direction_query.get_mut(parent.get()) {
                    if roam.timer.finished() {
                        target_direction.direction = default();
                    } else {
                        target_direction.direction = roam.direction;
                    }
                }
                if roam.timer.finished() {
                    *state = ActionState::Success;
                }
            }
            // All Actions should make sure to handle cancellations!
            // ActionState::Cancelled => {
            //     if let Ok(mut target_direction) = direction_query.get_mut(*actor) {
            //         target_direction.direction = default();
            //     }
            //     debug!("Action was cancelled. Considering this a failure.");
            //     *state = ActionState::Failure;
            // }
            _ => {}
        }
    }
}
