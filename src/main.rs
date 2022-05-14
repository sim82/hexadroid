use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_rapier2d::prelude::*;
use hexadroid::{droid::WeaponDirection, input::InputTarget};

fn main() {
    let mut app = App::new();

    // bevy plugins
    app.add_plugins(DefaultPlugins)
        .add_system(exit_on_esc_system);

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
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 100.0))
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(RigidBody::Fixed);

    commands
        .spawn()
        .insert(Collider::ball(32.0))
        .insert(Transform::from_xyz(200.0, 0.0, 0.0))
        .insert(hexadroid::input::InputTarget::default())
        .insert(ExternalForce::default())
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Friction {
            coefficient: 0.5,
            ..default()
        })
        .insert(Restitution {
            coefficient: 1.0,
            ..default()
        })
        .insert(hexadroid::droid::GroundFriction)
        .insert(Velocity::default())
        .insert(Name::new("droid"))
        .insert(WeaponDirection::default());

    // commands
    //     .spawn_bundle(hexadroid::droid::DroidBundle::default())
    //     .insert(InputTarget::default());
}
