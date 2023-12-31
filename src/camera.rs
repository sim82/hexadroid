use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
};
use bevy_mouse_tracking_plugin::{
    mouse_pos::{InitMouseTracking, InitWorldTracking},
    MainCamera,
};
// use bevy_mouse_tracking_plugin::prelude::*;

#[derive(Component, Default)]
pub struct CameraTarget;

fn setup_camera_system(mut commands: Commands) {
    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                tonemapping: Tonemapping::TonyMcMapface,
                ..default()
            },
            // BloomSettings::SCREEN_BLUR,
            BloomSettings::default(),
        ))
        .add(InitWorldTracking)
        .insert(MainCamera);
}

fn track_camera_system(
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<CameraTarget>)>,
    target_query: Query<&GlobalTransform, With<CameraTarget>>,
) {
    if let (Ok(mut camera_transform), Ok(target_transform)) =
        (camera_query.get_single_mut(), target_query.get_single())
    {
        debug!(
            "camera track: {:?} {:?}",
            target_transform.translation(),
            camera_transform.translation
        );
        let dist = target_transform.translation() - camera_transform.translation;
        let l = dist.length();
        const DEADZONE: f32 = 100.0;
        const OUTER: f32 = 200.0;
        const MAX_SPEED: f32 = 50.0;

        if l > DEADZONE {
            let dir = dist.normalize_or_zero();
            let v = ((l - DEADZONE).clamp(0.0, OUTER) / OUTER) * MAX_SPEED;
            camera_transform.translation += dir * v;
        }
    }
}
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera_system)
            .add_systems(Update, track_camera_system);
    }
}
