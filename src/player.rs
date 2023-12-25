use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerMarker;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {}
}
