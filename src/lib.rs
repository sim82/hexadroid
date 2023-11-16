// #![feature(array_zip)]

use bevy::{
    app::{AppExit, PluginGroupBuilder},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::EguiPlugin;
// use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use clap::Parser;
use hexagon_tiles::{
    layout::{Layout, LAYOUT_ORIENTATION_POINTY},
    point::Point,
};
use menu::MenuState;
use state::GameState;

use crate::{
    debug_ui::DebugUiPlugin, game::GamePlugin, hexton::HextonPlugin, menu::MenuPlugin,
    particle::ParticlePlugin, portal::PortalPlugin, ship::ShipPlugin, state::StatePlugin,
    weapon::WeaponPlugin,
};

pub mod collision;
pub mod droid;
pub mod input;
pub mod portal;
pub mod tiles;

pub mod hexton;
pub mod particle;
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
pub mod weapon;

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
pub mod debug_ui;
pub mod menu;
pub mod state;
pub mod game {
    use bevy::prelude::*;
    use bevy_prototype_lyon::{prelude::*, shapes};
    use hexagon_tiles::hexagon::Hex;
    use rand_distr::Normal;

    use crate::{
        camera::CameraTarget,
        droid::{ai::new_shooting_droid_ai, AiDroidBundle, DroidBundle, PlayerDroidBundle},
        hexton::{HextonBundle, HEXTON_VERTICES},
        input::InputTarget,
        particle::ColorGenerator,
        portal::Portal,
        prelude::*,
        ship::{ShipBundle, SHIP_VERTICES},
        state::GameState,
        CmdlineArgs,
    };

    pub struct GamePlugin;

    fn game_setup(mut commands: Commands, args: Res<CmdlineArgs>) {
        let shape = shapes::RegularPolygon {
            sides: 6,
            feature: shapes::RegularPolygonFeature::Radius(32.0),
            ..shapes::RegularPolygon::default()
        };

        let player = if args.ship {
            let ship_shape = shapes::Polygon {
                points: SHIP_VERTICES.into(),
                closed: true,
            };

            let ship_shape_builder = GeometryBuilder::build_as(&ship_shape);

            commands
                .spawn(ShipBundle::new("ship"))
                .insert(ShapeBundle {
                    path: ship_shape_builder,
                    spatial: SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
                        ..default()
                    },
                    ..default()
                })
                .insert(default_stroke(YELLOW_HDR))
                .insert(InputTarget)
                .insert(CameraTarget)
                .id()
        } else if args.hexton {
            let hexton_shape = shapes::Polygon {
                points: HEXTON_VERTICES.into(),
                closed: true,
            };

            let hexton_shape_builder = GeometryBuilder::build_as(&hexton_shape);

            commands
                .spawn(HextonBundle::new("hexton"))
                .insert(ShapeBundle {
                    path: hexton_shape_builder,
                    spatial: SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(100.0, 142.0, 0.0)),
                        ..default()
                    },
                    ..default()
                })
                .insert(default_stroke(BLUE_HDR))
                .insert(InputTarget)
                .insert(CameraTarget)
                .id()
        } else if args.benchmark {
            commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(100.0, 142.0, 0.0)),
                    ..default()
                })
                .insert(ParticleSource {
                    rate: 50,
                    direction: ParticleDirection::Uniform,
                    speed_distr: Normal::new(200.0, 90.0).unwrap(),
                    lifetime_distr: Normal::new(0.8, 0.5).unwrap(),
                    velocity_offset: Vec2::default(),
                    damping: default(),
                    initial_offset: 0.0,
                    color_generator: ColorGenerator::Static(7),
                })
                .insert(CameraTarget)
                .id()
            //
        } else {
            let my_shape_builder = GeometryBuilder::build_as(&shape);

            commands
                .spawn(DroidBundle::new("player", args.gravity))
                .insert(PlayerDroidBundle::default())
                .insert(ShapeBundle {
                    path: my_shape_builder,
                    spatial: SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
                        ..default()
                    },
                    ..default()
                })
                .insert(default_stroke(GREEN_HDR))
                // .insert(ParticleSource {
                //     rate: 1000,
                //     direction: ParticleDirection::Uniform,
                //     speed: 100.0,
                //     speed_spread: 50.0,
                //     lifetime: 1.0,
                //     lifetime_spread: 0.5,
                // })
                .id()
        };

        if !args.no_droid {
            let enemy_shape_builder = GeometryBuilder::build_as(&shape);

            commands
                .spawn(DroidBundle::new("r2d2", args.gravity))
                // .insert_bundle(AiDroidBundle::with_enemy(enemy))
                .insert(AiDroidBundle::with_enemy(player))
                .insert(ShapeBundle {
                    path: enemy_shape_builder,
                    spatial: SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(-100.0, 100.0, 0.0)),
                        ..default()
                    },
                    ..default()
                })
                .insert(default_stroke(RED_HDR))
                .insert(new_shooting_droid_ai());
            commands.spawn_empty().insert(Portal {
                tile_pos: TilePos(Hex::new(5, -1)),
                timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            });
        }
    }
    impl Plugin for GamePlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(OnEnter(GameState::Game), game_setup);
        }
    }
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

    #[clap(short = 'l', long)]
    pub diaglog: bool,

    #[clap(short = 'p', long)]
    pub benchmark: bool,
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
#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub enum Despawn {
    #[default]
    ThisFrame,
    TimeToLive(f32),
    FramesToLive(u32),
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
            Despawn::FramesToLive(ref mut f) => {
                if *f == 0 {
                    true
                } else {
                    *f -= 1;
                    false
                }
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
        app.add_systems(Update, toggle_on_esc_system)
            .add_systems(Last, despawn_reaper_system);
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
            .add(ShapePlugin)
            .add(waypoint::WaypointPlugin)
            .add(PortalPlugin)
            .add(ShipPlugin)
            .add(HextonPlugin)
            .add(ParticlePlugin)
            .add(WeaponPlugin)
            .add(DebugUiPlugin)
            .add(MenuPlugin)
            .add(GamePlugin)
            .add(StatePlugin);

        // egui plugins
        #[cfg(feature = "inspector")]
        let group = {
            if self.args.world_inspector {
                group.add(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
            } else {
                group.add(EguiPlugin)
            }
        };
        #[cfg(not(feature = "inspector"))]
        let group = group.add(EguiPlugin);

        let group = if self.args.worldbuild {
            group.add(worldbuild::WorldbuildPlugin)
        } else {
            group
        };
        let group = if self.args.debug_draw {
            group.add(RapierDebugRenderPlugin::default())
            // .add(DebugLinesPlugin::default())
        } else {
            group
        };
        if self.args.diaglog {
            group
                .add(FrameTimeDiagnosticsPlugin)
                .add(LogDiagnosticsPlugin::default())
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
pub fn toggle_on_esc_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
    cur_game_state: Res<State<GameState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        // app_exit_events.send_default();
        match cur_game_state.get() {
            GameState::Splash => todo!(),
            GameState::Menu => {}
            GameState::Game => {
                menu_state.set(MenuState::Main);
                game_state.set(GameState::Menu);
            }
        }
    }
}
pub mod prelude {
    pub use crate::collision::CollisionFxType;
    pub use crate::colors::*;
    pub use crate::particle::{ParticleDirection, ParticleSource};
    pub use crate::tiles::{TileCache, TilePos, TileType, TilesState};
    pub use crate::tunables::default_stroke;
    pub use crate::Despawn;
}
