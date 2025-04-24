use bevy::{color::palettes::css::CRIMSON, prelude::*};

use super::{frontend_utils::FrontendState, play_level::play_level};

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, States)]
enum MenuState {
    #[default]
    Main,
    Disabled,
    Settings,
}

#[derive(Component)]
struct OnMainMenu;

#[derive(Component)]
struct OnStatusMenu;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Exit,
    Settings,
    ToMainMenu,
}

#[derive(Component)]
struct SelectedOption;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

const TEXT_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

pub fn main_menu_plugin(app: &mut App) -> () {
    app.init_state::<MenuState>()
        .add_systems(OnEnter(FrontendState::MainMenu), menu_setup)
        .add_systems(OnEnter(FrontendState::MainMenu), menu_setup)
        .add_systems(OnEnter(MenuState::Main), main_menu_setup)
        .add_systems(OnExit(MenuState::Main), despawn_menu::<OnMainMenu>)
        .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
        .add_systems(OnExit(MenuState::Settings), despawn_menu::<OnStatusMenu>)
        .add_systems(OnEnter(FrontendState::LevelSelectMenu), setup_game)
        .add_systems(
            Update,
            (menu_action, button_colour_update).run_if(in_state(FrontendState::MainMenu)),
        );
}

fn button_colour_update(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) -> () {
    for (interaction, mut background_colour, selected) in &mut interaction_query {
        match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => {
                *background_colour = PRESSED_BUTTON.into();
            }
            (Interaction::Hovered, Some(_)) => {
                *background_colour = HOVERED_PRESSED_BUTTON.into();
            }
            (Interaction::Hovered, None) => {
                *background_colour = HOVERED_BUTTON.into();
            }
            (Interaction::None, None) => {
                *background_colour = NORMAL_BUTTON.into();
            }
        }
    }
}

fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) -> () {
    menu_state.set(MenuState::Main);
}

fn settings_menu_setup(mut commands: Commands) -> () {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnStatusMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(CRIMSON.into()),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Settings (Under Construction)"),
                        TextFont {
                            font_size: 67.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ));
                    parent
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::ToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn((Text::new("Main Menu"), button_text_style.clone()));
                        });
                });
        });
}
fn main_menu_setup(mut commands: Commands) -> () {
    // Common style for all buttons on the screen
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    #[allow(unused_variables)]
    let button_icon_node = Node {
        width: Val::Px(30.0),
        // This takes the icons out of the flexbox flow, to be positioned exactly
        position_type: PositionType::Absolute,
        // The icon will be close to the left border of the button
        left: Val::Px(10.0),
        ..default()
    };
    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnMainMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(CRIMSON.into()),
                ))
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn((
                        Text::new("Bevy Game Menu UI"),
                        TextFont {
                            font_size: 67.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ));

                    // Display three buttons for each action available from the main menu:
                    // - new game
                    // - settings
                    // - quit
                    parent
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Play,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("New Game"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    parent
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Settings,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Settings"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    parent
                        .spawn((
                            Button,
                            button_node,
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Exit,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Quit"),
                                button_text_font,
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        });
}

fn setup_game(
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<FrontendState>>,
) -> () {
    play_level();
    game_state.set(FrontendState::MainMenu);
    menu_state.set(MenuState::Main);
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<FrontendState>>,
) -> () {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Play => {
                    game_state.set(FrontendState::LevelSelectMenu);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::Exit => {
                    app_exit_events.send(AppExit::Success);
                }
                MenuButtonAction::Settings => {
                    menu_state.set(MenuState::Settings);
                }
                MenuButtonAction::ToMainMenu => {
                    menu_state.set(MenuState::Main);
                }
            }
        }
    }
}

fn despawn_menu<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) -> () {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
