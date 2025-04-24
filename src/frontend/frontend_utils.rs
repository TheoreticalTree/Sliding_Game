use bevy::{color::Color, state::state::States};

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

const SCREEN_WIDTH: u32 = 1600;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, States)]
pub enum FrontendState {
    #[default]
    MainMenu,
    LevelSelectMenu,
}
