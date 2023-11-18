use bevy::prelude::*;

// if true, GameState::None will automatically switch through to GameState::Game
// can be used for restarting game.
#[derive(Resource)]
pub struct AutoStart(pub bool);

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    None,
    Game,
    Paused,
}

fn auto_start_system(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut auto_start: ResMut<AutoStart>,
) {
    if *state.get() == GameState::None && auto_start.0 {
        auto_start.0 = false;
        next_state.set(GameState::Game);
    }
}
pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .insert_resource(AutoStart(false))
            .add_systems(PreUpdate, auto_start_system);
    }
}
