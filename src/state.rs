use bevy::prelude::*;
use bevy_rapier2d::plugin::RapierConfiguration;

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

#[derive(Component)]
#[component(storage = "SparseSet")]
pub enum GameDespawn {
    TimeToLive(f32, f32),
    FramesToLive(u32, u32),
}
impl Default for GameDespawn {
    fn default() -> Self {
        Self::FramesToLive(1, 1)
    }
}
impl GameDespawn {
    pub fn time_to_live(f: f32) -> Self {
        Self::TimeToLive(f, f)
    }
    pub fn frames_to_live(f: u32) -> Self {
        Self::FramesToLive(f, f)
    }

    pub fn get_f(&self) -> f32 {
        match self {
            GameDespawn::TimeToLive(ttl, initial_ttl) => {
                //
                ttl / initial_ttl
            }
            GameDespawn::FramesToLive(_, _) => 1.0, // TODO!
        }
        .clamp(0.0, 1.0)
    }
}
fn game_despawn_reaper_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut GameDespawn)>,
) {
    for (entity, mut despawn) in query.iter_mut() {
        let despawn = match *despawn {
            GameDespawn::TimeToLive(ref mut ttl, _) => {
                *ttl -= time.delta_seconds();
                *ttl <= 0.0
            }
            GameDespawn::FramesToLive(ref mut f, _) => {
                if *f == 0 {
                    true
                } else {
                    *f -= 1;
                    false
                }
            }
        };
        if despawn {
            trace!("game despawn {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
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
fn startup_system(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.physics_pipeline_active = false;
}
fn enter_game_state(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.physics_pipeline_active = true;
}

fn exit_game_state(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.physics_pipeline_active = false;
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .insert_resource(AutoStart(false))
            .add_systems(Startup, startup_system)
            .add_systems(PreUpdate, auto_start_system)
            .add_systems(OnEnter(GameState::Game), enter_game_state)
            .add_systems(OnExit(GameState::Game), exit_game_state)
            .add_systems(
                Last,
                game_despawn_reaper_system.run_if(in_state(GameState::Game)),
            );
    }
}
