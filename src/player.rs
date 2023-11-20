use crate::{
    camera::CameraTarget, droid::DroidMarker, input::InputTarget, prelude::*, ship::ShipMarker,
};
use bevy::prelude::*;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum PlayerState {
    #[default]
    None,
    Droid,
    Ship,
    Hexton,
}

#[derive(Component)]
pub struct PlayerMarker;

fn enter_droid(
    mut commands: Commands,
    query: Query<Entity, (With<PlayerMarker>, With<DroidMarker>)>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(InputTarget)
            .insert(CameraTarget);
    }
}
fn exit_droid(
    mut commands: Commands,
    query: Query<Entity, (With<PlayerMarker>, With<DroidMarker>, With<InputTarget>)>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .remove::<InputTarget>()
            .remove::<CameraTarget>();
    }
}
fn enter_ship(
    mut commands: Commands,
    query: Query<Entity, (With<PlayerMarker>, With<ShipMarker>)>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(InputTarget)
            .insert(CameraTarget);
    }
}
fn exit_ship(
    mut commands: Commands,
    query: Query<Entity, (With<PlayerMarker>, With<ShipMarker>, With<InputTarget>)>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .remove::<InputTarget>()
            .remove::<CameraTarget>();
    }
}
fn enter_hexton() {}
fn exit_hexton() {}

fn enter_none(mut player_state: ResMut<NextState<PlayerState>>) {
    player_state.set(PlayerState::None);
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<PlayerState>()
            .add_systems(OnEnter(PlayerState::Droid), enter_droid)
            .add_systems(OnExit(PlayerState::Droid), exit_droid)
            .add_systems(OnEnter(PlayerState::Ship), enter_ship)
            .add_systems(OnExit(PlayerState::Ship), exit_ship)
            .add_systems(OnEnter(PlayerState::Hexton), enter_hexton)
            .add_systems(OnExit(PlayerState::Hexton), exit_hexton)
            .add_systems(OnEnter(GameState::None), enter_none);
    }
}
