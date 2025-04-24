use bevy::prelude::*;

mod frontend_utils;
use frontend_utils::FrontendState;

mod main_menu;
use main_menu::main_menu_plugin;

mod play_level;

pub fn start_game_frontend() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<FrontendState>()
        .add_systems(Startup, init_camera)
        .add_plugins(main_menu_plugin)
        .run();
}

fn init_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
