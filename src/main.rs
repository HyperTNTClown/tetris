mod plugin;
mod render;
mod components;

use bevy::app::App;
use bevy::DefaultPlugins;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, plugin::Plugin))
        .run();
}

