use std::fmt::Debug;
use std::hash::Hash;
use bevy::app::{App, First, MainScheduleOrder, PostUpdate, Startup};
use bevy::ecs::schedule::{ExecutorKind, ScheduleLabel};
use bevy::prelude::{Component, FixedUpdate, Input, IntoSystemConfigs, KeyCode, Query, Res, ResMut, Resource, Schedule, Time, Timer, Update, World};
use bevy::time::{Fixed, TimerMode};
use bevy::utils::default;
use bevy::window::WindowRef::Entity;
use bevy::winit::{winit_runner, WinitWindows};
use winit::event_loop::EventLoop;
use winit::window::Window;
use crate::components::{Drawable, Updated};
use crate::render::{render, render_events, Renderer};

pub(crate) struct Plugin;

#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
struct Render;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let mut render_sched = Schedule::new(Render);
        render_sched.set_executor_kind(ExecutorKind::SingleThreaded);
        render_sched.add_systems(render_events);
        render_sched.add_systems(render.after(render_events));

        app
            .add_schedule(render_sched)
            .add_systems(Startup, setup_rendering)
            .add_systems(Update, move_piece)
            .set_runner(|mut app| {
                winit_runner(app)
            });

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(PostUpdate, Render);
    }
}

fn setup_rendering(mut world: &mut World) {
    let mut window_map = world.get_non_send_resource_mut::<WinitWindows>().unwrap();
    let mut window = window_map.windows.values().collect::<Vec<&Window>>().as_slice()[0];

    let renderer = Renderer::new(window);
    world.insert_resource(renderer);
    world.spawn(Drawable {
        shape: 2,
        position: [0., 0., 0.],
        shape_data: [0.5; 8],
        ..default()
    }).insert(Updated(true));
    world.spawn(Drawable {
        shape: 1,
        position: [-3., 0., 0.],
        shape_data: [2.; 8],
        ..default()
    }).insert(Updated(true));
    world.spawn(Drawable {
        shape: 1,
        position: [3., 0., 0.],
        shape_data: [2.; 8],
        ..default()
    }).insert(Updated(true));


    world.insert_resource(MovePieceTimer(Timer::from_seconds( 1.0, TimerMode::Repeating)));
}

#[derive(Resource)]
struct MovePieceTimer(Timer);

fn move_piece(mut query: Query<(&mut Drawable, &mut Updated)>, time: Res<Time>, mut timer: ResMut<MovePieceTimer>, input: Res<Input<KeyCode>>) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        for (mut drawable, mut updated) in query.iter_mut() {
            if !updated.0 {
                drawable.position[2] += 0.25;
                updated.0 = true;
            }
        }
    }

    if input.just_pressed(KeyCode::Space) {
        for (mut drawable, mut updated) in query.iter_mut() {
            if !updated.0 {
                drawable.position[2] += 0.25;
                updated.0 = true;
            }
        }
    }

    if input.just_pressed(KeyCode::Left) {
        for (mut drawable, mut updated) in query.iter_mut() {
            if !updated.0 {
                drawable.position[0] -= 1.;
                updated.0 = true;
            }
        }
    }

    if input.just_pressed(KeyCode::Right) {
        for (mut drawable, mut updated) in query.iter_mut() {
            if !updated.0 {
                drawable.position[0] += 1.;
                updated.0 = true;
            }
        }
    }

    if input.just_pressed(KeyCode::Down) {
        for (mut drawable, mut updated) in query.iter_mut() {
            if !updated.0 {
                drawable.position[1] -= 1.;
                updated.0 = true;
            }
        }
    }

    if input.just_pressed(KeyCode::Up) {
        for (mut drawable, mut updated) in query.iter_mut() {
            if !updated.0 {
                drawable.position[1] += 1.;
                updated.0 = true;
            }
        }
    }
}