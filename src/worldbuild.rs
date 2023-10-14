use crate::{
    camera::CameraTarget, hex_point_to_vec2, prelude::*, vec2_to_hex_point, Despawn, HEX_LAYOUT,
};
use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_egui::{egui, EguiContext};
use bevy_rapier2d::prelude::RapierContext;
#[allow(deprecated)]
use egui_extras::RetainedImage;
use hexagon_tiles::hexagon::{Hex, HexMath, HexRound};
use perlin2d::PerlinNoise2D;
use std::io::BufWriter;

#[derive(Copy, Clone, Eq, PartialEq)]
enum RebuildState {
    None,
    Despawn,
    Respawn,
}
#[derive(Resource)]
pub struct WorldState {
    min: Hex,
    max: Hex,

    pub min_target: Hex,
    pub max_target: Hex,

    center: Hex,

    perlin: PerlinNoise2D,

    rebuild: RebuildState,
    noise_preview: bool,
}

const NOISE_SCALE: f32 = 0.1;

const PERLIN_SCALE: f64 = 64.0;
const INITIAL_SIZE: i32 = 20;

impl Default for WorldState {
    fn default() -> Self {
        Self {
            min: Hex::new(0, 0),
            max: Hex::new(0, 0),
            min_target: Hex::new(-INITIAL_SIZE, -INITIAL_SIZE),
            max_target: Hex::new(INITIAL_SIZE, INITIAL_SIZE),
            center: Hex::new(0, 0),
            perlin: PerlinNoise2D::new(
                4,
                1.0,
                1.0,
                2.4,
                2.9,
                (PERLIN_SCALE, PERLIN_SCALE),
                -0.35,
                101,
            ),
            rebuild: RebuildState::None,
            noise_preview: false,
        }
    }
}

fn update_walls_noise(
    mut commands: Commands,
    _tiles_state: Res<TilesState>,
    tiles_cache: Res<TileCache>,
    mut world_state: ResMut<WorldState>,
    camera_query: Query<&Transform, With<CameraTarget>>,
) {
    if let Ok(Transform { translation, .. }) = camera_query.get_single() {
        let point = vec2_to_hex_point(translation.xy());
        let hex = hexagon_tiles::layout::LayoutTool::pixel_to_hex(HEX_LAYOUT, point).round();
        // info!("center: {:?}", hex);

        let dist = world_state.center.sub(hex);
        if dist.q().abs() >= 2 || dist.r().abs() >= 2 {
            info!("move center {:?}", hex);
            world_state.max_target = world_state.max_target.sub(dist);
            world_state.min_target = world_state.min_target.sub(dist);
            world_state.center = hex;
        }
    }

    if world_state.rebuild == RebuildState::None
        && world_state.max_target == world_state.max
        && world_state.min_target == world_state.min
    {
        return;
    }

    let valid_range_q = world_state.min_target.q()..=world_state.max_target.q();
    let valid_range_r = world_state.min_target.r()..=world_state.max_target.r();

    let old_range_q = world_state.min.q()..=world_state.max.q();
    let old_range_r = world_state.min.r()..=world_state.max.r();

    // if world_state.max_target < world_state.max {
    // shrink
    for (pos, entity) in tiles_cache.tiles.iter() {
        let q = pos.0.q();
        let r = pos.0.r();

        if world_state.rebuild == RebuildState::Despawn
            || !valid_range_q.contains(&q)
            || !valid_range_r.contains(&r)
        {
            commands.entity(*entity).insert(Despawn::ThisFrame);
        }
    }
    // } else {
    // grow
    for q in valid_range_q {
        for r in valid_range_r.clone() {
            // if r >= world_state.min
            //     && r <= world_state.max
            //     && q >= world_state.min
            //     && q <= world_state.max
            if world_state.rebuild != RebuildState::Respawn
                && old_range_q.contains(&q)
                && old_range_r.contains(&r)
            {
                continue;
            }
            let h = hexagon_tiles::hexagon::Hex::new(q, r);

            let p = hexagon_tiles::layout::LayoutTool::hex_to_pixel(HEX_LAYOUT, h);
            let p = hex_point_to_vec2(p) * NOISE_SCALE;
            let noise = world_state.perlin.get_noise(p.x.into(), p.y.into());
            // info!("noise {} {} {} {}", q, r, p, noise);

            if noise < 0.5 {
                continue;
            }

            let h = hexagon_tiles::hexagon::Hex::new(q, r);
            // if q.abs() != 5 && r.abs() != 5 {
            //     continue;
            // }

            let _entity = commands
                .spawn(SpatialBundle::default())
                .insert(TilePos(h))
                .insert(TileType {
                    wall: true,
                    immediate_collider: false,
                })
                .id();
            // commands.entity(tiles_state.tile_root).add_child(entity);
        }
    }
    // }
    world_state.min = world_state.min_target;
    world_state.max = world_state.max_target;
    world_state.rebuild = match world_state.rebuild {
        RebuildState::Despawn => RebuildState::Respawn,
        _ => RebuildState::None,
    };
}

#[allow(deprecated)]
fn worldbuid_egui_ui_system(
    mut egui_context: Query<&mut EguiContext>,
    mut world_state: ResMut<WorldState>,
    mut image: Local<Option<RetainedImage>>,
) {
    let mut amplitude = world_state.perlin.get_amplitude();
    let mut bias = world_state.perlin.get_bias();
    let mut frequency = world_state.perlin.get_frequency();
    let mut lacunarity = world_state.perlin.get_lacunarity();
    let mut octaves = world_state.perlin.get_octaves();
    let mut persistence = world_state.perlin.get_persistence();
    let mut scale = world_state.perlin.get_scale();

    let Ok(mut egui_context) = egui_context.get_single_mut() else {
        return;
    };
    egui::Window::new("path").show(egui_context.get_mut(), |ui| {
        egui::Grid::new("my_grid").num_columns(2).show(ui, |ui| {
            ui.label("amplitude");
            ui.add(egui::Slider::new(&mut amplitude, 0.0..=2.0));
            ui.end_row();

            ui.label("bias");
            ui.add(egui::Slider::new(&mut bias, -1.0..=1.0));
            ui.end_row();

            ui.label("frequency");
            ui.add(egui::Slider::new(&mut frequency, 0.0..=4.0));
            ui.end_row();

            ui.label("lacunarity");
            ui.add(egui::Slider::new(&mut lacunarity, 0.0..=4.0));
            ui.end_row();

            ui.label("octaves");
            ui.add(egui::Slider::new(&mut octaves, 1..=8));
            ui.end_row();

            ui.label("persistence");
            ui.add(egui::Slider::new(&mut persistence, 0.0..=3.0));
            ui.end_row();

            ui.label("scale x");
            ui.add(egui::Slider::new(&mut scale.0, 64.0..=1024.0));
            ui.end_row();

            ui.label("scale y");
            ui.add(egui::Slider::new(&mut scale.1, 64.0..=1024.0));
            ui.end_row();
        });

        if ui.button("rebuild").clicked() {
            world_state.rebuild = RebuildState::Despawn;
        }

        ui.checkbox(&mut world_state.noise_preview, "preview");

        if world_state.noise_preview {
            const SIZE: usize = 256;
            let mut color_image = egui::ColorImage::new([SIZE, SIZE], egui::Color32::GREEN);
            for y in 0..SIZE {
                for x in 0..SIZE {
                    let p = hexagon_tiles::layout::LayoutTool::hex_to_pixel(
                        HEX_LAYOUT,
                        world_state.center,
                    );
                    let p = hex_point_to_vec2(p) * NOISE_SCALE;

                    let offset_x = -((SIZE / 2) as f64) + p.x as f64;
                    let offset_y = -((SIZE / 2) as f64) + p.y as f64;

                    let noise = world_state
                        .perlin
                        .get_noise((x as f64) + offset_x, (y as f64) + offset_y)
                        .clamp(0.0, 1.0);
                    color_image.pixels[x + (SIZE - y - 1) * SIZE] =
                        egui::Color32::from_gray((noise * 255.0) as u8);
                }
            }

            let retained_image =
                egui_extras::RetainedImage::from_color_image("noise_preview", color_image);
            *image = Some(retained_image);
        }

        if let Some(image) = &*image {
            ui.image((image.texture_id(ui.ctx()), image.size_vec2()));
        }
    });

    world_state.perlin.set_amplitude(amplitude);
    world_state.perlin.set_bias(bias);
    world_state.perlin.set_frequency(frequency);
    world_state.perlin.set_lacunarity(lacunarity);
    world_state.perlin.set_octaves(octaves);
    world_state.perlin.set_persistence(persistence);
    world_state.perlin.set_scale(scale);
}

fn world_debug_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    mut world_state: ResMut<WorldState>,
) {
    // info!("scale: {:?}", rapier_config);

    const INCREMENT_I: i32 = 1;
    let increment = Hex::new(INCREMENT_I, INCREMENT_I);
    if keyboard_input.just_pressed(KeyCode::Y) {
        if world_state.max_target.q() > 1 {
            world_state.max_target = world_state.max_target.sub(increment);
            world_state.min_target = world_state.min_target.add(increment);
        }
    } else if keyboard_input.just_pressed(KeyCode::U) {
        if world_state.max_target.q() < 40 {
            world_state.max_target = world_state.max_target.add(increment);
            world_state.min_target = world_state.min_target.sub(increment);
        }
    } else if keyboard_input.just_pressed(KeyCode::Q) {
        if let Ok(file) = std::fs::File::create("physics.yaml") {
            let _ = serde_yaml::to_writer(BufWriter::new(file), &*rapier_context);
        }
    }
}

pub struct WorldbuildPlugin;

impl Plugin for WorldbuildPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldState>().add_systems(
            Update,
            (
                update_walls_noise,
                worldbuid_egui_ui_system,
                world_debug_input_system,
            ),
        );
    }
}
