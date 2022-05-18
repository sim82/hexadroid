#![feature(array_zip)]

use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLinesPlugin;
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
pub mod render;

pub const HEX_LAYOUT: Layout = Layout {
    orientation: LAYOUT_ORIENTATION_POINTY,
    size: Point { x: 64.0, y: 64.0 },
    origin: Point { x: 0.0, y: 0.0 },
};

pub fn hex_point_to_vec2(point: Point) -> Vec2 {
    Vec2::new(point.x as f32, point.y as f32)
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
            info!("despawn {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub struct DefaultPlugin;
impl Plugin for DefaultPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, despawn_reaper_system);
    }
}

pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        use bevy_rapier2d::prelude::*;

        group.add(DefaultPlugin);

        // bevy_rapier plugins
        group
            .add(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
            .add(RapierDebugRenderPlugin::default());

        group.add(DebugLinesPlugin::default());

        // egui plugins
        #[cfg(feature = "inspector")]
        {
            group.add(bevy_inspector_egui::WorldInspectorPlugin::new());
        }

        group
            .add(DefaultPlugin)
            .add(input::InputPlugin)
            .add(droid::DroidPlugin)
            .add(droid::ai::AiPlugin)
            .add(collision::CollisionPlugin)
            .add(camera::CameraPlugin)
            .add(tiles::TilesPlugin)
            .add(render::RenderPlugin);
    }
}
