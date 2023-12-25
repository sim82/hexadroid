use crate::{
    camera::CameraTarget,
    droid::{ai::new_shooting_droid_ai, DroidHealth, DroidMarker},
    input::InputTarget,
    prelude::*,
    ship::{ShipInput, ShipMarker},
};
use bevy::prelude::*;
use big_brain::thinker::ThinkerBuilder;

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
    query: Query<(Entity, &Transform), With<DroidMarker>>,
    ship_query: Query<&Transform, /*(With<PlayerMarker>,*/ With<ShipMarker>>,
) {
    let Ok(ship_transform) = ship_query.get_single() else {
        return;
    };
    let mut min_dist = f32::MAX;
    let mut enter_droid = None;
    for (entity, transform) in &query {
        let d = transform
            .translation
            .distance_squared(ship_transform.translation);
        if d < min_dist {
            enter_droid = Some(entity);
            min_dist = d;
        }
        // commands
        //     .entity(entity)
        //     .insert(PrimaryPlayerBundle::default());
    }
    if let Some(entity) = enter_droid {
        commands
            .entity(entity)
            .insert(PrimaryPlayerBundle::default())
            .insert(PlayerMarker)
            .insert(DroidHealth::default()) // crappy way to instantly heal the taken over droid. should clear emp load only
            .remove::<ThinkerBuilder>();
    }
}

#[allow(clippy::type_complexity)]
fn exit_droid(
    mut commands: Commands,
    query: Query<Entity, (With<PlayerMarker>, With<DroidMarker>, With<InputTarget>)>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .remove::<PrimaryPlayerBundle>()
            .remove::<PlayerMarker>()
            .insert(new_shooting_droid_ai());
    }
}

#[allow(clippy::type_complexity)]
fn enter_ship(
    mut commands: Commands,
    query: Query<Entity, /*(With<PlayerMarker>, */ With<ShipMarker>>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(PrimaryPlayerBundle::default())
            .insert(PlayerMarker);
    }
}

#[allow(clippy::type_complexity)]
fn exit_ship(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut ShipInput),
        (With<PlayerMarker>, With<ShipMarker>, With<InputTarget>),
    >,
) {
    for (entity, mut ship_input) in &mut query {
        commands
            .entity(entity)
            .remove::<PrimaryPlayerBundle>()
            .remove::<PlayerMarker>();
        *ship_input = ShipInput::default();
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
        app.add_state::<PlayerState>();
        // .add_systems(OnEnter(PlayerState::Droid), enter_droid)
        // .add_systems(OnExit(PlayerState::Droid), exit_droid)
        // .add_systems(OnEnter(PlayerState::Ship), enter_ship)
        // .add_systems(OnExit(PlayerState::Ship), exit_ship)
        // .add_systems(OnEnter(PlayerState::Hexton), enter_hexton)
        // .add_systems(OnExit(PlayerState::Hexton), exit_hexton)
        // .add_systems(OnEnter(GameState::None), enter_none);
    }
}
