use bevy::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes};
use bevy_rapier2d::prelude::*;
use clap::Parser;
use hexadroid::prelude::*;
use hexadroid::{
    camera::CameraTarget,
    droid::{ai::new_shooting_droid_ai, AiDroidBundle, PlayerDroidBundle},
    exit_on_esc_system,
    hexton::{HextonBundle, HEXTON_VERTICES},
    input::InputTarget,
    portal::Portal,
    ship::{ShipBundle, SHIP_VERTICES},
    CmdlineArgs,
};
use hexagon_tiles::hexagon::Hex;

fn main() {
    let args = CmdlineArgs::parse();

    let mut app = App::new();
    // bevy plugins
    app.add_plugins(DefaultPlugins)
        .add_systems(Update, exit_on_esc_system)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::default());

    app.add_plugins(hexadroid::DefaultPlugins::new(args.clone())); //default().with_debug_draw(args.debug_draw));

    let gravity = if args.gravity {
        Vec2::Y * -9.81 * 50.0
    } else {
        Vec2::ZERO
    };
    app.insert_resource(RapierConfiguration {
        gravity,
        ..default()
    });

    app.add_systems(Startup, setup_geometry)
        // .add_startup_system(setup_linedraw_test)
        // .add_startup_system(setup_lyon_test)
        ;
    app.insert_resource(args);

    app.run();
}

fn setup_geometry(mut commands: Commands, args: Res<CmdlineArgs>) {
    let shape = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(32.0),
        ..shapes::RegularPolygon::default()
    };

    let player = if args.ship {
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
            .insert(InputTarget)
            .insert(CameraTarget)
            .id()
    } else if args.hexton {
        let hexton_shape = shapes::Polygon {
            points: HEXTON_VERTICES.into(),
            closed: true,
        };

        let hexton_shape_builder = GeometryBuilder::build_as(&hexton_shape);

        commands
            .spawn(HextonBundle::new("hexton"))
            .insert(ShapeBundle {
                path: hexton_shape_builder,
                spatial: SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(100.0, 142.0, 0.0)),
                    ..default()
                },
                ..default()
            })
            .insert(default_stroke(BLUE_HDR))
            .insert(InputTarget)
            .insert(CameraTarget)
            .id()
    } else {
        let my_shape_builder = GeometryBuilder::build_as(&shape);

        commands
            .spawn(hexadroid::droid::DroidBundle::new("player", args.gravity))
            .insert(PlayerDroidBundle::default())
            .insert(ShapeBundle {
                path: my_shape_builder,
                spatial: SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
                    ..default()
                },
                ..default()
            })
            .insert(default_stroke(GREEN_HDR))
            // .insert(ParticleSource {
            //     rate: 1000,
            //     direction: ParticleDirection::Uniform,
            //     speed: 100.0,
            //     speed_spread: 50.0,
            //     lifetime: 1.0,
            //     lifetime_spread: 0.5,
            // })
            .id()
    };

    if !args.no_droid {
        let enemy_shape_builder = GeometryBuilder::build_as(&shape);

        commands
            .spawn(hexadroid::droid::DroidBundle::new("r2d2", args.gravity))
            // .insert_bundle(AiDroidBundle::with_enemy(enemy))
            .insert(AiDroidBundle::with_enemy(player))
            .insert(ShapeBundle {
                path: enemy_shape_builder,
                spatial: SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(-100.0, 100.0, 0.0)),
                    ..default()
                },
                ..default()
            })
            .insert(default_stroke(RED_HDR))
            .insert(new_shooting_droid_ai());
    }

    commands.spawn_empty().insert(Portal {
        tile_pos: TilePos(Hex::new(5, -1)),
        timer: Timer::from_seconds(2.0, TimerMode::Repeating),
    });
}
