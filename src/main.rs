use bevy::{
    diagnostic::DiagnosticsPlugin,
    //  input::system::exit_on_esc_system,
    prelude::*,
};
use bevy_prototype_lyon::{prelude::*, shapes};
use bevy_rapier2d::prelude::*;
use clap::Parser;
use hexadroid::{
    droid::{ai::new_shooting_droid_ai, AiDroidBundle, PlayerDroidBundle},
    exit_on_esc_system,
    portal::Portal,
    tiles::TilePos,
    CmdlineArgs,
};
use hexagon_tiles::hexagon::Hex;

fn main() {
    let args = CmdlineArgs::parse();

    let mut app = App::new();
    // bevy plugins
    app.add_plugins(DefaultPlugins)
        .add_plugin(DiagnosticsPlugin)
        .add_system(exit_on_esc_system)
        .insert_resource(Msaa::default());

    app.add_plugins(hexadroid::DefaultPlugins::default().with_debug_draw(args.debug_draw));

    let gravity = if args.gravity {
        Vec2::Y * -9.81 * 50.0
    } else {
        Vec2::ZERO
    };
    app.insert_resource(RapierConfiguration {
        gravity,
        ..default()
    });

    // egui plugins
    #[cfg(feature = "inspector")]
    {
        if args.world_inspector {
            app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
        }
    }

    app.add_startup_system(setup_geometry)
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

    let my_shape_builder = GeometryBuilder::build_as(
        &shape,
        DrawMode::Stroke(StrokeMode::new(Color::GREEN, 10.0)),
        Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
    );

    let enemy = commands
        .spawn_bundle(hexadroid::droid::DroidBundle::new("player", args.gravity))
        .insert_bundle(PlayerDroidBundle::default())
        .insert_bundle(my_shape_builder)
        .id();

    if !args.no_droid {
        let enemy_shape_builder = GeometryBuilder::build_as(
            &shape,
            DrawMode::Stroke(StrokeMode::new(Color::RED, 10.0)),
            Transform::from_translation(Vec3::new(-100.0, 100.0, 0.0)),
        );

        commands
            .spawn_bundle(hexadroid::droid::DroidBundle::new("r2d2", args.gravity))
            .insert_bundle(AiDroidBundle::with_enemy(enemy))
            .insert_bundle(enemy_shape_builder)
            .insert(new_shooting_droid_ai());
    }

    commands.spawn().insert(Portal {
        tile_pos: TilePos(Hex::new(5, -1)),
        timer: Timer::from_seconds(2.0, true),
    });
}
