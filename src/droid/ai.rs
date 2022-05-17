use bevy::{ecs::component, math::Vec3Swizzles, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier2d::prelude::*;
use hexagon_tiles::{hexagon::HEX_DIRECTIONS, layout::LayoutTool};

use crate::{droid::weapon::PROJECTILE_SPEED, hex_point_to_vec2, HEX_LAYOUT};

use super::{AttackRequest, WeaponDirection, WeaponState};

#[derive(Component, Default)]
pub struct AssaultAi {}
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct PrimaryEnemy {
    pub enemy: Entity,
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

fn assault_predict_system(
    mut commands: Commands,
    enemy_query: Query<(&Transform, &Velocity)>,
    mut debug_lines: ResMut<DebugLines>,
    mut assault_query: Query<
        (
            &Transform,
            &PrimaryEnemy,
            &mut AttackRequest,
            &mut WeaponDirection,
            &WeaponState,
        ),
        With<AssaultAi>,
    >,
) {
    for (
        Transform {
            translation: my_translation,
            ..
        },
        PrimaryEnemy { enemy },
        mut attack_request,
        mut weapon_direction,
        weapon_state,
    ) in assault_query.iter_mut()
    {
        if weapon_state.reload_timeout > f32::EPSILON {
            // currently reloading. cannot fire. clear previous attack request.
            attack_request.primary_attack = false;
            continue;
        }

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
            let enemy_dir = enemy_velocity.normalize_or_zero();
            let enemy_speed = enemy_velocity.length();

            if enemy_speed <= f32::EPSILON {
                // enemy not moving
                continue;
            }
            const SQRT2_2: f32 = std::f32::consts::SQRT_2 / 2.0;
            let directions = [
                Vec2::new(1.0, 0.0),
                Vec2::new(SQRT2_2, -SQRT2_2),
                Vec2::new(0.0, -1.0),
                Vec2::new(-SQRT2_2, -SQRT2_2),
                Vec2::new(-1.0, 0.0),
                Vec2::new(-SQRT2_2, SQRT2_2),
                Vec2::new(0.0, 1.0),
                Vec2::new(SQRT2_2, SQRT2_2),
            ];
            // Vec2(1.0, 2.0);
            // for dir in HEX_DIRECTIONS.iter().map(|dir| {
            //     hex_point_to_vec2(LayoutTool::hex_to_pixel(HEX_LAYOUT, *dir)).normalize_or_zero()
            // }) {
            for dir in directions {
                // find intersection between predicted projectile and enemy trajectories
                let enemy_line = Line(
                    enemy_translation.xy(),
                    enemy_translation.xy() + *enemy_velocity,
                );
                let projectile_line = Line(
                    my_translation.xy() + dir * 50.0,
                    my_translation.xy() + dir * PROJECTILE_SPEED,
                );
                debug_lines.line(enemy_line.0.extend(0.0), enemy_line.1.extend(0.0), 0.0);
                debug_lines.line(
                    projectile_line.0.extend(0.0),
                    projectile_line.1.extend(0.0),
                    0.0,
                );

                if let Some(intersect) = projectile_line.intersect2(enemy_line) {
                    // predicted 'time to intersection'
                    let my_d = (intersect - my_translation.xy()).length();
                    let enemy_d = (intersect - enemy_translation.xy()).length();

                    let my_t = my_d / PROJECTILE_SPEED;
                    let enemy_t = enemy_d / enemy_speed;

                    // if projectile and enemy are predicted to reach intersection at roughly the
                    // same time, shoot in this direction.
                    const MAX_DELTA: f32 = 0.1;
                    if (my_t - enemy_t).abs() < MAX_DELTA {
                        info!("shoot");
                        attack_request.primary_attack = true;
                        weapon_direction.direction = dir;
                    } else {
                        attack_request.primary_attack = false;
                    }
                }
            }
        }
    }
}

pub struct AiPlugin;
impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(assault_predict_system);
    }
}

#[derive(Bundle, Default)]
pub struct AssaultAiBundle {
    assault_ai: AssaultAi,
}

// impl AssaultAiBundle {
//     pub fn new() -> Self {
//         Self {
//             assault_ai: AssaultAi {},
//         }
//     }
// }
