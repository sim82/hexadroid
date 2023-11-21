use crate::{
    camera::CameraTarget, droid::DroidMarker, input::InputTarget, prelude::*, ship::ShipMarker,
};
use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct PrimaryPlayerBundle {
    input_target: InputTarget,
    camera_target: CameraTarget,
}

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

#[allow(clippy::type_complexity)]
fn enter_droid(
    mut commands: Commands,
    query: Query<Entity, (With<PlayerMarker>, With<DroidMarker>)>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(PrimaryPlayerBundle::default());
    }
}

#[allow(clippy::type_complexity)]
fn exit_droid(
    mut commands: Commands,
    query: Query<Entity, (With<PlayerMarker>, With<DroidMarker>, With<InputTarget>)>,
) {
    for entity in &query {
        commands.entity(entity).remove::<PrimaryPlayerBundle>();
    }
}

#[allow(clippy::type_complexity)]
fn enter_ship(
    mut commands: Commands,
    query: Query<Entity, (With<PlayerMarker>, With<ShipMarker>)>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(PrimaryPlayerBundle::default());
    }
}

#[allow(clippy::type_complexity)]
fn exit_ship(
    mut commands: Commands,
    query: Query<Entity, (With<PlayerMarker>, With<ShipMarker>, With<InputTarget>)>,
) {
    for entity in &query {
        commands.entity(entity).remove::<PrimaryPlayerBundle>();
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
