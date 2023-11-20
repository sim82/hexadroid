use bevy::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes};
use hexagon_tiles::hexagon::Hex;
use rand_distr::Normal;

use crate::{
    camera::CameraTarget,
    droid::{ai::new_shooting_droid_ai, AiDroidBundle, DroidBundle, PlayerDroidBundle},
    hexton::{HextonBundle, HEXTON_VERTICES},
    input::InputTarget,
    particle::ColorGenerator,
    player::{PlayerMarker, PlayerState},
    portal::Portal,
    prelude::*,
    ship::{ShipBundle, SHIP_VERTICES},
    state::GameState,
};

#[derive(Resource)]
pub struct GameSpawnInfo {
    pub spawn_player_ship: bool,
    pub spawn_player_droid: bool,
    pub spawn_player_jnr: bool,
    pub spawn_enemy_droids: u32,
    pub spawn_benchmark: bool,
    pub gravity: bool,
}

#[derive(Component)]
pub struct GameMarker;

fn game_setup(
    mut commands: Commands,
    spawn_info: Res<GameSpawnInfo>,
    mut player_state: ResMut<NextState<PlayerState>>,
) {
    let shape = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(32.0),
        ..shapes::RegularPolygon::default()
    };

    let ship_shape = shapes::Polygon {
        points: SHIP_VERTICES.into(),
        closed: true,
    };

    let ship_shape_builder = GeometryBuilder::build_as(&ship_shape);

    commands
        .spawn(ShipBundle::new("ship"))
        .insert(ShapeBundle {
            path: ship_shape_builder,
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
                ..default()
            },
            ..default()
        })
        .insert(default_stroke(YELLOW_HDR))
        // .insert(InputTarget)
        // .insert(CameraTarget)
        .insert(GameMarker)
        .insert(PlayerMarker)
        .id();

    let my_shape_builder = GeometryBuilder::build_as(&shape);

    let player = commands
        .spawn(DroidBundle::new("player", spawn_info.gravity))
        // .insert(PlayerDroidBundle::default())
        .insert(ShapeBundle {
            path: my_shape_builder,
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
                ..default()
            },
            ..default()
        })
        .insert(default_stroke(GREEN_HDR))
        .insert(GameMarker)
        .insert(PlayerMarker)
        // .insert(ParticleSource {
        //     rate: 1000,
        //     direction: ParticleDirection::Uniform,
        //     speed: 100.0,
        //     speed_spread: 50.0,
        //     lifetime: 1.0,
        //     lifetime_spread: 0.5,
        // })
        .id();

    player_state.set(PlayerState::Droid);
    // let player = if spawn_info.spawn_player_ship {
    //     let ship_shape = shapes::Polygon {
    //         points: SHIP_VERTICES.into(),
    //         closed: true,
    //     };

    //     let ship_shape_builder = GeometryBuilder::build_as(&ship_shape);

    //     commands
    //         .spawn(ShipBundle::new("ship"))
    //         .insert(ShapeBundle {
    //             path: ship_shape_builder,
    //             spatial: SpatialBundle {
    //                 transform: Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
    //                 ..default()
    //             },
    //             ..default()
    //         })
    //         .insert(default_stroke(YELLOW_HDR))
    //         .insert(InputTarget)
    //         .insert(CameraTarget)
    //         .insert(GameMarker)
    //         .id()
    // } else if spawn_info.spawn_player_jnr {
    //     let hexton_shape = shapes::Polygon {
    //         points: HEXTON_VERTICES.into(),
    //         closed: true,
    //     };

    //     let hexton_shape_builder = GeometryBuilder::build_as(&hexton_shape);

    //     commands
    //         .spawn(HextonBundle::new("hexton"))
    //         .insert(ShapeBundle {
    //             path: hexton_shape_builder,
    //             spatial: SpatialBundle {
    //                 transform: Transform::from_translation(Vec3::new(100.0, 142.0, 0.0)),
    //                 ..default()
    //             },
    //             ..default()
    //         })
    //         .insert(default_stroke(BLUE_HDR))
    //         .insert(InputTarget)
    //         .insert(CameraTarget)
    //         .insert(GameMarker)
    //         .id()
    // } else if spawn_info.spawn_benchmark {
    //     commands
    //         .spawn(SpatialBundle {
    //             transform: Transform::from_translation(Vec3::new(100.0, 142.0, 0.0)),
    //             ..default()
    //         })
    //         .insert(ParticleSource {
    //             rate: 50,
    //             direction: ParticleDirection::Uniform,
    //             speed_distr: Normal::new(200.0, 90.0).unwrap(),
    //             lifetime_distr: Normal::new(0.8, 0.5).unwrap(),
    //             velocity_offset: Vec2::default(),
    //             damping: default(),
    //             initial_offset: 0.0,
    //             color_generator: ColorGenerator::Static(7),
    //         })
    //         .insert(CameraTarget)
    //         .insert(GameMarker)
    //         .id()
    //     //
    // } else if spawn_info.spawn_player_droid {
    //     let my_shape_builder = GeometryBuilder::build_as(&shape);

    //     commands
    //         .spawn(DroidBundle::new("player", spawn_info.gravity))
    //         .insert(PlayerDroidBundle::default())
    //         .insert(ShapeBundle {
    //             path: my_shape_builder,
    //             spatial: SpatialBundle {
    //                 transform: Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
    //                 ..default()
    //             },
    //             ..default()
    //         })
    //         .insert(default_stroke(GREEN_HDR))
    //         .insert(GameMarker)
    //         // .insert(ParticleSource {
    //         //     rate: 1000,
    //         //     direction: ParticleDirection::Uniform,
    //         //     speed: 100.0,
    //         //     speed_spread: 50.0,
    //         //     lifetime: 1.0,
    //         //     lifetime_spread: 0.5,
    //         // })
    //         .id()
    // } else {
    //     panic!("dont know what to spawn");
    // };

    let mut enemy_offset = Vec3::new(-100.0, 100.0, 0.0);
    for _ in 0..spawn_info.spawn_enemy_droids {
        let enemy_shape_builder = GeometryBuilder::build_as(&shape);

        commands
            .spawn(DroidBundle::new("r2d2", spawn_info.gravity))
            // .insert_bundle(AiDroidBundle::with_enemy(enemy))
            .insert(AiDroidBundle::with_enemy(player))
            .insert(ShapeBundle {
                path: enemy_shape_builder,
                spatial: SpatialBundle {
                    transform: Transform::from_translation(enemy_offset),
                    ..default()
                },
                ..default()
            })
            .insert(default_stroke(RED_HDR))
            .insert(GameMarker)
            .insert(new_shooting_droid_ai());
        enemy_offset.x += 100.0;
        commands.spawn_empty().insert(Portal {
            tile_pos: TilePos(Hex::new(5, -1)),
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        });
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::None), game_setup)
            .add_systems(OnEnter(GameState::None), despawn_screen::<GameMarker>);
    }
}
