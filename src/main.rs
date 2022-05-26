use bevy::{input::system::exit_on_esc_system, prelude::*, sprite::MaterialMesh2dBundle};
use bevy_prototype_lyon::{
    prelude::{tess::FillTessellator, *},
    shapes,
};
use bevy_rapier2d::prelude::*;
use clap::Parser;
use hexadroid::{
    camera::CameraTarget,
    droid::{ai::PrimaryEnemy, WeaponDirection},
    input::InputTarget,
    render::MyMaterial,
    tiles::{TilePos, TileType},
    CmdlineArgs, HEX_LAYOUT,
};
use hexagon_tiles::layout::LayoutTool;

fn main() {
    let args = CmdlineArgs::parse();

    let mut app = App::new();
    // bevy plugins
    app.add_plugins(DefaultPlugins)
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
    // commands
    //     .spawn()
    //     .insert(Collider::cuboid(100.0, 100.0))
    //     .insert(Transform::from_xyz(0.0, 0.0, 0.0))
    //     .insert(RigidBody::Fixed);

    let shape = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(32.0),
        ..shapes::RegularPolygon::default()
    };

    let my_shape_builder = GeometryBuilder::build_as(
        &shape,
        DrawMode::Stroke(StrokeMode::new(Color::GREEN, 10.0)),
        // fill_mode: bevy_prototype_lyon::draw::FillMode::color(Color::CYAN),
        // outline_mode: StrokeMode::new(Color::BLACK, 2.0),
        // },
        Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
    );

    let enemy = commands
        .spawn_bundle(hexadroid::droid::DroidBundle::new("player", args.gravity))
        .insert(InputTarget::default())
        .insert(CameraTarget)
        .insert_bundle(my_shape_builder)
        .id();

    let enemy_shape_builder = GeometryBuilder::build_as(
        &shape,
        DrawMode::Stroke(StrokeMode::new(Color::RED, 10.0)),
        // fill_mode: bevy_prototype_lyon::draw::FillMode::color(Color::CYAN),
        // outline_mode: StrokeMode::new(Color::BLACK, 2.0),
        // },
        Transform::from_translation(Vec3::new(-100.0, 100.0, 0.0)),
    );

    commands
        .spawn_bundle(hexadroid::droid::DroidBundle::new("r2d2", args.gravity))
        .insert_bundle(hexadroid::droid::ai::AssaultAiBundle::default())
        .insert(PrimaryEnemy { enemy })
        .insert_bundle(enemy_shape_builder);
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
            material: my_material_assets.add(MyMaterial {
                alpha: 0.5,
                color: Color::RED,
            }),
            ..default()
        })
        .insert(Name::new("quad"));
}

fn setup_lyon_test(mut commands: Commands) {
    let shape = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(200.0),
        ..shapes::RegularPolygon::default()
    };

    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Stroke(StrokeMode::new(Color::GREEN, 10.0)),
        // fill_mode: bevy_prototype_lyon::draw::FillMode::color(Color::CYAN),
        // outline_mode: StrokeMode::new(Color::BLACK, 2.0),
        // },
        Transform::default(),
    ));
}
