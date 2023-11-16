use bevy::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes};
use bevy_rapier2d::prelude::*;
use clap::Parser;
use hexadroid::particle::ColorGenerator;
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
use rand_distr::Normal;

fn main() {
    let args = CmdlineArgs::parse();

    let mut app = App::new();
    // bevy plugins
    app.add_plugins(DefaultPlugins)
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

    app.insert_resource(args);

    app.run();
}
