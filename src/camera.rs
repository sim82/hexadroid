use bevy::{prelude::*, render::camera::Camera2d};

#[derive(Component)]
pub struct CameraTarget;

fn setup_camera_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn track_camera_system(
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<CameraTarget>)>,
    target_query: Query<&Transform, With<CameraTarget>>,
) {
    if let (Ok(mut camera_transform), Ok(target_transform)) =
        (camera_query.get_single_mut(), target_query.get_single())
    {
        let dist = target_transform.translation - camera_transform.translation;
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
        app.add_startup_system(setup_camera_system)
            .add_system(track_camera_system);
    }
}
