use bevy::{app::AppExit, prelude::*};

use crate::{game::GameSpawnInfo, prelude::*, state::AutoStart};
// Tag component used to tag entities added on the main menu screen
#[derive(Component)]
struct OnMainMenuScreen;

// Tag component used to tag entities added on the settings menu screen
#[derive(Component)]
struct OnSettingsMenuScreen;

// Tag component used to tag entities added on the display settings menu screen
#[derive(Component)]
struct OnDisplaySettingsMenuScreen;

// Tag component used to tag entities added on the sound settings menu screen
#[derive(Component)]
struct OnSoundSettingsMenuScreen;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
// const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
// const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
// const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
// All actions that can be triggered from a button click

#[allow(dead_code)]
#[derive(Component)]
enum MenuButtonAction {
    PlayDroid,
    PlayShip,
    PlayHexton,
    DropGame,
    // Settings,
    // SettingsDisplay,
    // SettingsSound,
    // BackToMainMenu,
    // BackToSettings,
    Quit,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum MenuState {
    #[default]
    Main,
    Disabled,
}
// fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
//     menu_state.set(MenuState::Main);
// }

fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Common style for all buttons on the screen
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_icon_style = Style {
        width: Val::Px(30.0),
        // This takes the icons out of the flexbox flow, to be positioned exactly
        position_type: PositionType::Absolute,
        // The icon will be close to the left border of the button
        left: Val::Px(10.0),
        ..default()
    };
    let font = asset_server.load("fonts/MonaspaceKrypton-Bold.otf");
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        // color: colors::COLORS[1],
        font: font.clone(),
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnMainMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        border: UiRect::px(10., 10., 10., 10.),
                        ..default()
                    },
                    background_color: Color::BLACK.with_a(0.5).into(),
                    // border_color: colors::COLORS[1].into(),
                    border_color: Color::GREEN.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn(
                        TextBundle::from_section(
                            "Hexxadroid",
                            TextStyle {
                                font_size: 80.0,
                                color: TEXT_COLOR,
                                font,
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        }),
                    );

                    // Display three buttons for each action available from the main menu:
                    // - new game
                    // - settings
                    // - quit
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::PlayDroid,
                        ))
                        .with_children(|parent| {
                            let icon = asset_server.load("textures/Game Icons/right.png");
                            parent.spawn(ImageBundle {
                                style: button_icon_style.clone(),
                                image: UiImage::new(icon),
                                ..default()
                            });
                            parent.spawn(TextBundle::from_section(
                                "Play Droid",
                                button_text_style.clone(),
                            ));
                        });
                    // parent
                    //     .spawn((
                    //         ButtonBundle {
                    //             style: button_style.clone(),
                    //             background_color: NORMAL_BUTTON.into(),
                    //             ..default()
                    //         },
                    //         MenuButtonAction::PlayShip,
                    //     ))
                    //     .with_children(|parent| {
                    //         let icon = asset_server.load("textures/Game Icons/right.png");
                    //         parent.spawn(ImageBundle {
                    //             style: button_icon_style.clone(),
                    //             image: UiImage::new(icon),
                    //             ..default()
                    //         });
                    //         parent.spawn(TextBundle::from_section(
                    //             "Play Ship",
                    //             button_text_style.clone(),
                    //         ));
                    //     });
                    // parent
                    //     .spawn((
                    //         ButtonBundle {
                    //             style: button_style.clone(),
                    //             background_color: NORMAL_BUTTON.into(),
                    //             ..default()
                    //         },
                    //         MenuButtonAction::PlayHexton,
                    //     ))
                    //     .with_children(|parent| {
                    //         let icon = asset_server.load("textures/Game Icons/right.png");
                    //         parent.spawn(ImageBundle {
                    //             style: button_icon_style.clone(),
                    //             image: UiImage::new(icon),
                    //             ..default()
                    //         });
                    //         parent.spawn(TextBundle::from_section(
                    //             "Play Cmdr",
                    //             button_text_style.clone(),
                    //         ));
                    //     });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::DropGame,
                        ))
                        .with_children(|parent| {
                            let icon = asset_server.load("textures/Game Icons/right.png");
                            parent.spawn(ImageBundle {
                                style: button_icon_style.clone(),
                                image: UiImage::new(icon),
                                ..default()
                            });
                            parent.spawn(TextBundle::from_section(
                                "Drop game",
                                button_text_style.clone(),
                            ));
                        });
                    // parent
                    //     .spawn((
                    //         ButtonBundle {
                    //             style: button_style.clone(),
                    //             background_color: NORMAL_BUTTON.into(),
                    //             ..default()
                    //         },
                    //         MenuButtonAction::Settings,
                    //     ))
                    //     .with_children(|parent| {
                    //         let icon = asset_server.load("textures/Game Icons/wrench.png");
                    //         parent.spawn(ImageBundle {
                    //             style: button_icon_style.clone(),
                    //             image: UiImage::new(icon),
                    //             ..default()
                    //         });
                    //         parent.spawn(TextBundle::from_section(
                    //             "Settings",
                    //             button_text_style.clone(),
                    //         ));
                    //     });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::Quit,
                        ))
                        .with_children(|parent| {
                            let icon = asset_server.load("textures/Game Icons/exitRight.png");
                            parent.spawn(ImageBundle {
                                style: button_icon_style,
                                image: UiImage::new(icon),
                                ..default()
                            });
                            parent.spawn(TextBundle::from_section("Quit", button_text_style));
                        });
                });
        });
}

#[allow(clippy::type_complexity)]
fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    cur_game_state: Res<State<GameState>>,
    mut auto_start: ResMut<AutoStart>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut spawn_info: ResMut<GameSpawnInfo>,
) {
    let mut start = false;
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => app_exit_events.send(AppExit),
                MenuButtonAction::PlayDroid => {
                    spawn_info.spawn_player_droid = true;
                    spawn_info.spawn_player_ship = false;
                    spawn_info.spawn_player_jnr = false;
                    start = true;
                }
                MenuButtonAction::PlayShip => {
                    spawn_info.spawn_player_droid = false;
                    spawn_info.spawn_player_ship = true;
                    spawn_info.spawn_player_jnr = false;
                    start = true;
                }
                MenuButtonAction::PlayHexton => {
                    spawn_info.spawn_player_droid = false;
                    spawn_info.spawn_player_ship = false;
                    spawn_info.spawn_player_jnr = true;
                    start = true;
                }
                MenuButtonAction::DropGame => {
                    game_state.set(GameState::None);
                    menu_state.set(MenuState::Main);
                } // MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                  // MenuButtonAction::SettingsDisplay => {
                  //     menu_state.set(MenuState::SettingsDisplay);
                  // }
                  // MenuButtonAction::SettingsSound => {
                  //     menu_state.set(MenuState::SettingsSound);
                  // }
                  // MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                  // MenuButtonAction::BackToSettings => {
                  //     menu_state.set(MenuState::Settings);
                  // }
            }
        }
    }
    if start {
        if *cur_game_state.get() != GameState::None {
            auto_start.0 = true;
            game_state.set(GameState::None);
        } else {
            game_state.set(GameState::Game);
        }
        menu_state.set(MenuState::Disabled);
    }
}
// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MenuState>()
            // .add_systems(OnEnter(GameState::Menu), menu_setup)
            // Systems to handle the main menu screen
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
            .add_systems(
                Update,
                menu_action,
                // (menu_action/*, button_system*/.run_if(in_state(GameState::Menu)),
            );
    }
}
