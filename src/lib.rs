// #![feature(array_zip)]

use bevy::{
    app::{AppExit, PluginGroupBuilder},
    prelude::*,
};
use bevy_egui::EguiPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use clap::Parser;
use hexagon_tiles::{
    layout::{Layout, LAYOUT_ORIENTATION_POINTY},
    point::Point,
};

use crate::{hexton::HextonPlugin, portal::PortalPlugin, ship::ShipPlugin};

pub mod collision;
pub mod droid;
pub mod input;
pub mod portal;
pub mod tiles;

pub mod hexton;
pub mod ship;
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

pub mod tunables {
    use bevy::prelude::Color;
    use bevy_prototype_lyon::prelude::{LineJoin, Stroke, StrokeOptions};

    pub const LINE_WIDTH: f32 = 4.0;

    pub const STROKE_OPTIONS: StrokeOptions = StrokeOptions::DEFAULT
        .with_line_width(LINE_WIDTH)
        .with_line_join(LineJoin::Round);
    pub fn default_stroke(color: Color) -> Stroke {
        Stroke {
            color,
            options: STROKE_OPTIONS,
        }
    }
}

pub mod collision_groups {
    use bevy_rapier2d::prelude::Group;

    pub const DROIDS: Group = Group::GROUP_1;
    pub const PROJECTILES: Group = Group::GROUP_2;
    pub const LEVEL: Group = Group::GROUP_3;
}

#[derive(Parser, Debug, Resource, Clone)]
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

    #[clap(short, long)]
    pub ship: bool,

    #[clap(short = 'x', long)]
    pub hexton: bool,

    #[clap(short = 'b', long)]
    pub worldbuild: bool,
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

pub mod colors {
    use bevy::prelude::Color;

    pub const COLORS_L: f32 = 3.75;

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

    pub const GREEN_HDR: Color = Color::rgb(0.0, 5.0, 0.0);
    pub const BLUE_HDR: Color = Color::rgb(0.0, 0.0, 5.0);
    pub const RED_HDR: Color = Color::rgb(5.0, 0.0, 0.0);
    pub const YELLOW_HDR: Color = Color::rgb(3.0, 3.0, 0.0);
}
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
        app.add_systems(Last, despawn_reaper_system);
    }
}

// #[derive(Default)]
pub struct DefaultPlugins {
    // debug_draw: bool,
    args: CmdlineArgs,
}

impl DefaultPlugins {
    // pub fn with_debug_draw(mut self, b: bool) -> Self {
    //     self.debug_draw = b;
    //     self
    // }

    pub fn new(args: CmdlineArgs) -> DefaultPlugins {
        DefaultPlugins { args }
    }
}

impl PluginGroup for DefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        use bevy_rapier2d::prelude::*;

        let group = PluginGroupBuilder::start::<Self>()
            .add(DefaultPlugin)
            // bevy_rapier plugins
            .add(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add(input::InputPlugin)
            .add(droid::DroidPlugin)
            .add(droid::ai::AiPlugin)
            .add(collision::CollisionPlugin)
            .add(camera::CameraPlugin)
            .add(tiles::TilesPlugin)
            // .add(render::RenderPlugin)
            // .add(render::pipeline::RenderShapePlugin)
            .add(ShapePlugin)
            // .add(worldbuild::WorldbuildPlugin)
            .add(waypoint::WaypointPlugin)
            .add(EguiPlugin)
            .add(PortalPlugin)
            .add(ShipPlugin)
            .add(HextonPlugin);
        let group = if self.args.worldbuild {
            group.add(worldbuild::WorldbuildPlugin)
        } else {
            group
        };
        if self.args.debug_draw {
            group
                .add(RapierDebugRenderPlugin::default())
                .add(DebugLinesPlugin::default())
        } else {
            group
        }
    }
}

pub fn exit_on_esc_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_events.send_default();
    }
}
pub mod prelude {
    pub use crate::colors::*;
    pub use crate::tiles::{TileCache, TilePos, TileType, TilesState};
    pub use crate::tunables::default_stroke;
    pub use crate::Despawn;
}
