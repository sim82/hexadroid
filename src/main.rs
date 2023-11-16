use bevy::prelude::*;

use bevy_rapier2d::prelude::*;
use clap::Parser;

use hexadroid::CmdlineArgs;

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
