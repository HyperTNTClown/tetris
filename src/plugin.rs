use crate::components::{BufferUpdate, Locked, Position, Tetr, TetrisGame, Tetromino, Updated};
use crate::render::{render, render_events, Renderer};
use bevy::app::{App, MainScheduleOrder, PostUpdate, Startup};
use bevy::ecs::schedule::{ExecutorKind, ScheduleLabel};
use bevy::prelude::*;
use bevy::tasks::block_on;
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
            .insert_resource(BufferUpdate(false))
            .set_runner(winit_runner);

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(PostUpdate, Render);
    }
}

fn setup_rendering(world: &mut World) {
    let window_map = world.get_non_send_resource::<WinitWindows>().unwrap();
    let window = window_map
        .windows
        .values()
        .collect::<Vec<&Window>>()
        .as_slice()[0];

    let mut renderer = block_on(Renderer::new(window));
    renderer.resize(window.inner_size().to_logical(1.0));
    world.insert_non_send_resource(renderer);

    world.insert_resource(MovePieceTimer(Timer::from_seconds(
        0.5,
        TimerMode::Repeating,
    )));

    info!("Rendering is set up!");
}

#[derive(Resource)]
struct MovePieceTimer(Timer);

fn move_piece(
    mut query: Query<(&mut Tetr, &mut Updated), Without<Locked>>,
    game: ResMut<TetrisGame>,
    time: Res<Time>,
    mut timer: ResMut<MovePieceTimer>,
    input: Res<Input<KeyCode>>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        for (mut tetr, mut updated) in query.iter_mut() {
            if !updated.0 {
                tetr.positions.iter_mut().for_each(|p| p.y -= 1);
                updated.0 = true;
            }
        }
    }

    // FIXME: Fix moving into other pieces sideways
    if input.just_pressed(KeyCode::Left) {
        for (mut tetr, mut updated) in query.iter_mut() {
            let pos = tetr.positions.clone();
            tetr.positions.iter_mut().for_each(|p| p.x -= 1);
            if tetr.positions.iter().any(|p| p.x < 0) {
                tetr.positions = pos;
            }
            updated.0 = true;
        }
    }

    if input.just_pressed(KeyCode::Right) {
        for (mut tetr, mut updated) in query.iter_mut() {
            let pos = tetr.positions.clone();
            tetr.positions.iter_mut().for_each(|p| p.x += 1);
            if tetr.positions.iter().any(|p| p.x > 9) {
                tetr.positions = pos;
            }
            updated.0 = true;
        }
    }

    if input.just_pressed(KeyCode::Down) {
        for (mut tetr, mut updated) in query.iter_mut() {
            tetr.positions.iter_mut().for_each(|p| p.y -= 1);
            updated.0 = true;
        }
    }

    if input.just_pressed(KeyCode::Up) {
        for (mut tetr, _) in query.iter_mut() {
            tetr.spin();
        }
    }

    if input.just_pressed(KeyCode::Space) {
        // Move piece all the way down until it hits something
        for (mut tetr, mut updated) in query.iter_mut() {
            while !check_field_under(&game, &tetr.positions) {
                tetr.positions.iter_mut().for_each(|p| p.y -= 1);
            }
            updated.0 = true;
        }
    }
}

fn spawn_new_piece(mut commands: Commands, query: Query<(&mut Tetr, &mut Updated, Entity), Without<Locked>>, mut rand: ResMut<GlobalRng>) {
    if query.is_empty() {
        let tetromino = match rand.u8(0..7) {
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

fn check_field_under(game: &TetrisGame, positions: &[Position]) -> bool {
    positions.iter().any(|p| p.y == 0 || game.field[p.y as usize - 1usize][p.x as usize])
}

fn update_board(mut game: ResMut<TetrisGame>, mut tetr: Query<&mut Tetr, With<Locked>>, mut buffer_update: ResMut<BufferUpdate>) {
    for position in tetr.iter().flat_map(|t| t.positions.iter()) {
        game.field[position.y as usize][position.x as usize] = true;
    }

    // remove full rows
    // TODO: Shift the rows above down
    let mut removed_rows = Vec::new();
    let mut row = 0;
    while row < game.field.len() {
        if game.field[row].iter().all(|&b| b) {
            game.field[row] = [false; 10];
            for mut tetr in &mut tetr {
                tetr.positions.retain(|p| p.y != row as u32 as i32);
            }
            buffer_update.0 = true;
            removed_rows.push(row);
        }
        row += 1;
    }

    for (i, e) in removed_rows.iter_mut().enumerate() {
        let e = *e - i;
        tetr.for_each_mut(|mut tetr| {
            tetr.positions.iter_mut().for_each(|p| {
                if p.y > (e as i32) {
                    p.y -= 1;
                }
            });
        });
    }

    if !removed_rows.is_empty() {
        game.field = [[false; 10]; 40];
    }

    // We need to remove the locked drawables to make it work...
    // And then there also is the buffer where we have written to sequentially,
    // so we actually either need to shift inside the buffer or just
    // simply overwrite it completely (performance vs simplicity)
    // TODO: implement this (probably just overwrite the buffer each time... for now)
    //       - Shouldn't be too performance heavy, as we only have to do this when a row is cleared
}

// TODO: might need some work as the player might want to slide the piece to the left or right when touching the ground
fn lock_pieces(mut commands: Commands, query: Query<(Entity, &Tetr), Without<Locked>>, game: Res<TetrisGame>) {
    for (entity, tetr) in query.iter() {
        if check_field_under(&game, &tetr.positions) {
            commands.get_entity(entity).unwrap().insert(Locked);
        }
    }
}