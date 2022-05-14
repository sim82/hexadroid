use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_rapier2d::prelude::*;
use hexadroid::{droid::WeaponDirection, input::InputTarget, HEX_LAYOUT};
use hexagon_tiles::layout::LayoutTool;

fn main() {
    let mut app = App::new();

    // bevy plugins
    app.add_plugins(DefaultPlugins)
        .add_system(exit_on_esc_system)
        .insert_resource(Msaa::default());

    // bevy_rapier plugins
    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        });

    // egui plugins
    #[cfg(feature = "inspector")]
    {
        app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
    }

    app.add_startup_system(setup_camera)
        .add_startup_system(setup_geometry);

    app.add_plugin(hexadroid::input::InputPlugin)
        .add_plugin(hexadroid::droid::DroidPlugin);

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn setup_geometry(mut commands: Commands) {
    // commands
    //     .spawn()
    //     .insert(Collider::cuboid(100.0, 100.0))
    //     .insert(Transform::from_xyz(0.0, 0.0, 0.0))
    //     .insert(RigidBody::Fixed);

    commands
        .spawn_bundle(hexadroid::droid::DroidBundle::with_name("player"))
        .insert(InputTarget::default());

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
