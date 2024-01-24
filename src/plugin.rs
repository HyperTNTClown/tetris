use crate::components::{Drawable, Locked, Position, Tetr, TetrisGame, Tetromino, Updated};
use crate::render::{render, render_events, Renderer};
use bevy::app::{App, MainScheduleOrder, PostUpdate, Startup};
use bevy::ecs::schedule::{ExecutorKind, ScheduleLabel};
use bevy::prelude::{Commands, Entity, Input, IntoSystemConfigs, KeyCode, Last, Query, Res, ResMut, Resource, Schedule, Time, Timer, Update, With, Without, World};
use bevy::time::TimerMode;
use bevy::winit::{winit_runner, WinitWindows};
use bevy_turborand::{DelegatedRng, GlobalRng};
use bevy_turborand::prelude::{RngPlugin};
use winit::window::Window;

pub(crate) struct Plugin;

#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
struct Render;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let mut render_sched = Schedule::new(Render);
        render_sched.set_executor_kind(ExecutorKind::SingleThreaded);
        render_sched.add_systems(render_events);
        render_sched.add_systems(render.after(render_events));

        app.add_schedule(render_sched)
            .add_plugins(RngPlugin::default())
            .add_systems(Startup, setup_rendering)
            .add_systems(Update, move_piece)
            .add_systems(PostUpdate, update_board)
            .add_systems(PostUpdate, spawn_new_piece)
            .add_systems(Last, lock_pieces)
            .insert_resource(TetrisGame::default())
            .set_runner(|mut app| winit_runner(app));

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(PostUpdate, Render);
    }
}

fn setup_rendering(mut world: &mut World) {
    let mut window_map = world.get_non_send_resource_mut::<WinitWindows>().unwrap();
    let mut window = window_map
        .windows
        .values()
        .collect::<Vec<&Window>>()
        .as_slice()[0];

    let renderer = Renderer::new(window);
    world.insert_resource(renderer);
    world.spawn(Tetr::new(Tetromino::O)).insert(Updated(true));

    world.insert_resource(MovePieceTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
}

#[derive(Resource)]
struct MovePieceTimer(Timer);

fn move_piece(
    mut query: Query<(&mut Tetr, &mut Updated), Without<Locked>>,
    mut game: ResMut<TetrisGame>,
    time: Res<Time>,
    mut timer: ResMut<MovePieceTimer>,
    input: Res<Input<KeyCode>>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        for (mut tetr, mut updated) in query.iter_mut() {
            if !updated.0 {
                //let field_under = game.field[drawable.position[0] as usize].get(drawable.position[1] as usize - 1);
                // We should not have to check if field is of invalid index, as we should lock all pieces that are at the bottom beforehand
                //let field_under = field_under.unwrap();
                tetr.positions.iter_mut().for_each(|p| p.y -= 1);
                println!("Move piece {:?}", tetr.positions[0]);
                updated.0 = true;
            }
        }
    }

    if input.just_pressed(KeyCode::Left) {
        for (mut tetr, mut updated) in query.iter_mut() {
            if !updated.0 {
                tetr.positions.iter_mut().for_each(|p| p.x -= 1);
                updated.0 = true;
            }
        }
    }

    if input.just_pressed(KeyCode::Right) {
        for (mut tetr, mut updated) in query.iter_mut() {
            if !updated.0 {
                tetr.positions.iter_mut().for_each(|p| p.x += 1);
                updated.0 = true;
            }
        }
    }

    if input.just_pressed(KeyCode::Down) {
        for (mut tetr, mut updated) in query.iter_mut() {
            if !updated.0 {
                tetr.positions.iter_mut().for_each(|p| p.y -= 1);
                updated.0 = true;
            }
        }
    }

    if input.just_pressed(KeyCode::Up) {
        for (mut tetr, mut updated) in query.iter_mut() {
            if !updated.0 {
                tetr.positions.iter_mut().for_each(|p| p.y += 1);
                updated.0 = true;
            }
        }
    }
}

fn spawn_new_piece(mut commands: Commands, mut query: Query<(&mut Tetr, &mut Updated, Entity), Without<Locked>>, mut rand: ResMut<GlobalRng>) {
    if query.is_empty() {
        let num = rand.u8(0..7);
        let tetromino = match num {
            0 => Tetromino::I,
            1 => Tetromino::O,
            2 => Tetromino::T,
            3 => Tetromino::S,
            4 => Tetromino::Z,
            5 => Tetromino::J,
            6 => Tetromino::L,
            _ => unreachable!()
        };
        commands.spawn(Tetr::new(tetromino)).insert(Updated(true));
    }
}

fn check_field_under(game: &TetrisGame, positions: &Vec<Position>) -> bool {
    positions.iter().any(|p| p.y == 0 || game.field[p.y as usize - 1usize][p.x as usize])
}

fn update_board(mut game: ResMut<TetrisGame>, tetr: Query<&Tetr, With<Locked>>) {
    for position in tetr.iter().flat_map(|t| t.positions.iter()) {
        game.field[position.y as usize][position.x as usize] = true;
    }
}

// TODO: might need some work as the player might want to slide the piece to the left or right when touching the ground
fn lock_pieces(mut commands: Commands, query: Query<(Entity, &Tetr), Without<Locked>>, mut game: Res<TetrisGame>) {
    for (entity, tetr) in query.iter() {
        if check_field_under(&game, &tetr.positions) {
            commands.get_entity(entity).unwrap().insert(Locked);
        }
    }
}