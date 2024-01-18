mod plugin;
mod render;
mod components;

use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::prelude::Schedules;
use bevy::winit::WinitPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, plugin::Plugin))
        .run();
}

