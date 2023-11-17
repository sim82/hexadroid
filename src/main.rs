use bevy::prelude::*;

use bevy_rapier2d::prelude::*;
use clap::Parser;

use hexadroid::{game::GameSpawnInfo, CmdlineArgs};

fn main() {
    let args = CmdlineArgs::parse();

    let spawn_info = GameSpawnInfo {
        spawn_player_ship: args.ship,
        spawn_player_droid: !args.ship && !args.hexton,
        spawn_player_jnr: args.hexton,
        spawn_enemy_droids: if args.no_droid { 0 } else { 1 },
        spawn_benchmark: args.benchmark,
        gravity: args.gravity,
    };
    let mut app = App::new();
    // bevy plugins
    app.add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(spawn_info)
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
