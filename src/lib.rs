#![feature(array_zip)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use clap::Parser;
use hexagon_tiles::{
    layout::{Layout, LAYOUT_ORIENTATION_POINTY},
    point::Point,
};

pub mod collision;
pub mod droid;
pub mod input;
pub mod tiles;
// pub mod worm {
//     use perlin_noise::PerlinNoise;

//     #[test]
//     fn test() {
//         let mut perlin_noise = PerlinNoise::new();
//         println!("{}", perlin_noise.get3d([1.0, 2.0, 3.0]));
//     }
// }
pub mod camera;
pub mod debug;
// pub mod render;
pub mod worldbuild;

pub mod waypoint;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CmdlineArgs {
    #[clap(short, long)]
    pub debug_draw: bool,

    #[clap(short, long)]
    pub empty: bool,

    #[clap(short, long)]
    pub gravity: bool,

    #[clap(short, long)]
    pub world_inspector: bool,

    #[clap(short, long)]
    pub no_droid: bool,
}

pub const HEX_LAYOUT: Layout = Layout {
    orientation: LAYOUT_ORIENTATION_POINTY,
    size: Point { x: 64.0, y: 64.0 },
    origin: Point { x: 0.0, y: 0.0 },
};

pub fn hex_point_to_vec2(point: Point) -> Vec2 {
    Vec2::new(point.x as f32, point.y as f32)
}

pub fn vec2_to_hex_point(v: Vec2) -> Point {
    Point {
        x: v.x.into(),
        y: v.y.into(),
    }
}

pub const COLORS_L: f32 = 0.75;

pub const COLORS: [Color; 12] = [
    Color::hsl(0.0, 1.0, COLORS_L),
    Color::hsl(30.0, 1.0, COLORS_L),
    Color::hsl(60.0, 1.0, COLORS_L),
    Color::hsl(90.0, 1.0, COLORS_L),
    Color::hsl(120.0, 1.0, COLORS_L),
    Color::hsl(150.0, 1.0, COLORS_L),
    Color::hsl(180.0, 1.0, COLORS_L),
    Color::hsl(210.0, 1.0, COLORS_L),
    Color::hsl(240.0, 1.0, COLORS_L),
    Color::hsl(270.0, 1.0, COLORS_L),
    Color::hsl(300.0, 1.0, COLORS_L),
    Color::hsl(330.0, 1.0, COLORS_L),
];

#[derive(Component)]
#[component(storage = "SparseSet")]
pub enum Despawn {
    ThisFrame,
    TimeToLive(f32),
}

pub fn despawn_reaper_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Despawn)>,
) {
    for (entity, mut despawn) in query.iter_mut() {
        let despawn = match *despawn {
            Despawn::ThisFrame => true,
            Despawn::TimeToLive(ref mut ttl) => {
                *ttl -= time.delta_seconds();
                *ttl <= 0.0
            }
        };
        if despawn {
            trace!("despawn {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub struct DefaultPlugin;
impl Plugin for DefaultPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Last, despawn_reaper_system);
    }
}

#[derive(Default)]
pub struct DefaultPlugins {
    debug_draw: bool,
}

impl DefaultPlugins {
    pub fn with_debug_draw(mut self, b: bool) -> Self {
        self.debug_draw = b;
        self
    }
}

impl PluginGroup for DefaultPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        use bevy_rapier2d::prelude::*;

        group.add(DefaultPlugin);

        // bevy_rapier plugins
        group.add(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));

        if self.debug_draw {
            group.add(RapierDebugRenderPlugin::default());
            group.add(DebugLinesPlugin::default());
        }

        group
            .add(input::InputPlugin)
            .add(droid::DroidPlugin)
            .add(droid::ai::AiPlugin)
            .add(collision::CollisionPlugin)
            .add(camera::CameraPlugin)
            .add(tiles::TilesPlugin)
            // .add(render::RenderPlugin)
            // .add(render::pipeline::RenderShapePlugin)
            .add(ShapePlugin)
            .add(worldbuild::WorldbuildPlugin)
            .add(waypoint::WaypointPlugin)
            .add(EguiPlugin);
    }
}
