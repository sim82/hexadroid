use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_rapier2d::prelude::*;
use hexadroid::{
    camera::CameraTarget,
    droid::{ai::PrimaryEnemy, WeaponDirection},
    input::InputTarget,
    HEX_LAYOUT,
};
use hexagon_tiles::layout::LayoutTool;

fn main() {
    let mut app = App::new();

    // bevy plugins
    app.add_plugins(DefaultPlugins)
        .add_system(exit_on_esc_system)
        .insert_resource(Msaa::default());

    app.add_plugins(hexadroid::DefaultPlugins);
    app.insert_resource(RapierConfiguration {
        gravity: Vec2::ZERO,
        ..default()
    });

    app.add_startup_system(setup_geometry);

    app.run();
}

fn setup_geometry(mut commands: Commands) {
    // commands
    //     .spawn()
    //     .insert(Collider::cuboid(100.0, 100.0))
    //     .insert(Transform::from_xyz(0.0, 0.0, 0.0))
    //     .insert(RigidBody::Fixed);

    let enemy = commands
        .spawn_bundle(hexadroid::droid::DroidBundle::with_name(
            Vec2::new(100.0, 100.0),
            "player",
        ))
        .insert(InputTarget::default())
        .insert(CameraTarget)
        .id();

    commands
        .spawn_bundle(hexadroid::droid::DroidBundle::with_name(
            Vec2::new(-100.0, 100.0),
            "r2d2",
        ))
        .insert_bundle(hexadroid::droid::ai::AssaultAiBundle::default())
        .insert(PrimaryEnemy { enemy });

    let h = hexagon_tiles::hexagon::Hex::new(0, 0);
    let corners = LayoutTool::polygon_corners(HEX_LAYOUT, h)
        .iter()
        .map(|p| Vec2::new(p.x as f32, p.y as f32))
        .collect();

    commands
        .spawn()
        .insert(Collider::polyline(
            corners,
            Some(vec![[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 0]]),
        ))
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(RigidBody::Fixed);
}
