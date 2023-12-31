use bevy::prelude::*;
use bevy_mouse_tracking_plugin::{MousePos, MousePosWorld};

fn mouse_input_system(mouse_pos: Res<MousePosWorld>) {
    info!("mouse pos: {:?}", mouse_pos);
}

pub struct EditPlugin;
impl Plugin for EditPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_input_system);
    }
}
