use bevy::{input::system::exit_on_esc_system, prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;
use hexadroid::{
    camera::CameraTarget,
    droid::{ai::PrimaryEnemy, WeaponDirection},
    input::InputTarget,
    render::MyMaterial,
    tiles::{TilePos, TileType},
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

    app.add_startup_system(setup_geometry)
        .add_startup_system(setup_linedraw_test);

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

    for q in -5..=5 {
        for r in -5..=5 {
            let h = hexagon_tiles::hexagon::Hex::new(q, r);

            if q.abs() != 5 && r.abs() != 5 {
                continue;
            }

            commands
                .spawn()
                .insert(TilePos(h))
                .insert(TileType { wall: true });

            // let corners = LayoutTool::polygon_corners(HEX_LAYOUT, h)
            //     .iter()
            //     .map(|p| Vec2::new(p.x as f32, p.y as f32))
            //     .collect();

            // commands
            //     .spawn()
            //     .insert(Collider::polyline(
            //         corners,
            //         Some(vec![[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 0]]),
            //     ))
            //     .insert(Transform::from_xyz(0.0, 0.0, 0.0))
            //     .insert(RigidBody::Fixed);
        }
    }
}

fn setup_linedraw_test(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut my_material_assets: ResMut<Assets<MyMaterial>>,
) {
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_assets
                .add(Mesh::from(shape::Quad::new(Vec2::new(100.0, 100.0))))
                .into(),
            material: my_material_assets.add(MyMaterial {}),
            ..default()
        })
        .insert(Name::new("quad"));
}
