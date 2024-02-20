use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

#[derive(Resource)]
pub struct BufferUpdate(pub(crate) bool);

#[derive(Component)]
pub struct Updated(pub(crate) bool);

#[derive(Component)]
pub struct Locked;

#[derive(Resource, Default)]
pub struct Glitch(pub(crate) f32);


// FIXME: MAYBE SPLIT SHAPE_DATA INTO TWO VEC4s

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Score {
    pub score: u32,
    pub level: u32,
}

impl Score {
    pub fn increase(&mut self, cleared_lines: u32) -> bool {
        match cleared_lines {
            1 => self.score += 1,
            2 => self.score += 3,
            3 => self.score += 5,
            4 => self.score += 8,
            _ => {}
        }

        if self.score >= self.goal() {
            self.score = 0;
            self.level += 1;
            return true;
        }
        false
    }

    pub fn goal(&self) -> u32 {
        5 * (self.level + 1)
    }

    pub fn timer(&self) -> Timer {
        match self.level {
            0 => Timer::from_seconds(1.0, TimerMode::Repeating),
            1 => Timer::from_seconds(0.79300, TimerMode::Repeating),
            2 => Timer::from_seconds(0.61780, TimerMode::Repeating),
            3 => Timer::from_seconds(0.47273, TimerMode::Repeating),
            4 => Timer::from_seconds(0.35520, TimerMode::Repeating),
            5 => Timer::from_seconds(0.26200, TimerMode::Repeating),
            6 => Timer::from_seconds(0.18968, TimerMode::Repeating),
            7 => Timer::from_seconds(0.13473, TimerMode::Repeating),
            8 => Timer::from_seconds(0.09388, TimerMode::Repeating),
            9 => Timer::from_seconds(0.06415, TimerMode::Repeating),
            10 => Timer::from_seconds(0.04298, TimerMode::Repeating),
            11 => Timer::from_seconds(0.02822, TimerMode::Repeating),
            12 => Timer::from_seconds(0.01815, TimerMode::Repeating),
            13 => Timer::from_seconds(0.01144, TimerMode::Repeating),
            14 => Timer::from_seconds(0.00706, TimerMode::Repeating),
            15 => Timer::from_seconds(0.00426, TimerMode::Repeating),
            16 => Timer::from_seconds(0.00252, TimerMode::Repeating),
            17 => Timer::from_seconds(0.00146, TimerMode::Repeating),
            18 => Timer::from_seconds(0.00082, TimerMode::Repeating),
            19 | _ => Timer::from_seconds(0.00046, TimerMode::Repeating),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Component)]
pub struct Drawable {
    pub position: [f32; 4],
    pub shape_data: [f32; 8],
}

impl Drawable {
    #[allow(dead_code)]
    pub fn new(x: isize, y: isize, z: isize, shape: Option<u32>) -> Self {
        let shape = shape.unwrap_or(0);
        let mut shape_data = [0.0; 8];
        shape_data[7] = shape as f32;
        Drawable {
            position: [x as f32, y as f32, z as f32, 0.0],
            shape_data,
        }
    }

    pub fn with_shape_data(x: isize, y: isize, z: isize, mut shape_data: [f32; 8], shape: Option<u32>) -> Self {
        let shape = shape.unwrap_or(0);
        shape_data[7] = shape as f32;
        Drawable {
            position: [x as f32, y as f32, z as f32, 0.0],
            shape_data,
        }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Default for Drawable {
    fn default() -> Self {
        Drawable {
            position: [0.0; 4],
            shape_data: [0.0; 8],
        }
    }
}

unsafe impl bytemuck::Zeroable for Drawable {}

unsafe impl bytemuck::Pod for Drawable {}

#[derive(Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug)]
pub enum Rotation {
    Zero = 0,
    Ninety = 1,
    OneEighty = 2,
    TwoHundredSeventy = 3,
}

// https://tetris.fandom.com/wiki/Tetris_Guideline
// https://tetris.fandom.com/wiki/SRS
// We workin' by the Guidelines
// Therefore no creativity is needed

#[derive(Resource)]
pub struct TetrisGame {
    /// Playfield is 10×40, where rows above 20 are hidden or obstructed by the field frame to trick the player into thinking it's 10×20.
    /// | Guidelines
    pub field: [[bool; 10]; 40],
    pub next: Option<Tetromino>,
    pub hold: Option<Tetromino>,
    pub score: Score,
    pub level: u32,
}

impl Default for TetrisGame {
    fn default() -> Self {
        TetrisGame {
            field: [[false; 10]; 40],
            next: None,
            hold: None,
            score: Score::default(),
            level: 0,
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub enum Tetromino {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Display for Tetromino {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tetromino::I => write!(f, "I"),
            Tetromino::O => write!(f, "O"),
            Tetromino::T => write!(f, "T"),
            Tetromino::S => write!(f, "S"),
            Tetromino::Z => write!(f, "Z"),
            Tetromino::J => write!(f, "J"),
            Tetromino::L => write!(f, "L"),
        }
    }
}

impl Tetromino {
    /// Cyan I,
    /// Yellow O,
    /// Purple T,
    /// Green S,
    /// Red Z,
    /// Blue J,
    /// Orange L
    /// | Guidelines
    ///
    /// RGB format, no need for alpha
    pub fn color(&self) -> [f32; 3] {
        match self {
            Tetromino::I => [0.0, 1.0, 1.0],
            Tetromino::O => [1.0, 1.0, 0.0],
            Tetromino::T => [1.0, 0.0, 1.0],
            Tetromino::S => [0.0, 1.0, 0.0],
            Tetromino::Z => [1.0, 0.0, 0.0],
            Tetromino::J => [0.0, 0.0, 1.0],
            Tetromino::L => [1.0, 0.5, 0.0],
        }
    }

    /// Tetromino start locations
    /// The I and O spawn in the middle columns
    /// The rest spawn in the left-middle columns
    /// The tetriminoes spawn horizontally with J, L and T spawning flat-side first.
    /// Spawn above playfield, row 21 for I, and 21/22 for all other tetriminoes.
    /// Immediately drop one space if no existing Block is in its path
    /// | Guidelines
    // TODO: we can potentially switch the vec for a custom struct, but that's for later
    //       Okay, maybe we should, as that would be wayyyy easier to work with,
    //       but that's for later
    pub fn start_positions(&self) -> Vec<Position> {
        match self {
            Tetromino::I => {
                vec![
                    Position { x: 3, y: 21 },
                    Position { x: 4, y: 21 },
                    Position { x: 5, y: 21 },
                    Position { x: 6, y: 21 },
                ]
            }
            Tetromino::O => {
                vec![
                    Position { x: 4, y: 21 },
                    Position { x: 5, y: 21 },
                    Position { x: 4, y: 22 },
                    Position { x: 5, y: 22 },
                ]
            }
            Tetromino::T => {
                vec![
                    Position { x: 3, y: 21 },
                    Position { x: 4, y: 21 },
                    Position { x: 5, y: 21 },
                    Position { x: 4, y: 22 },
                ]
            }
            Tetromino::S => {
                vec![
                    Position { x: 3, y: 21 },
                    Position { x: 4, y: 21 },
                    Position { x: 4, y: 22 },
                    Position { x: 5, y: 22 },
                ]
            }
            Tetromino::Z => {
                vec![
                    Position { x: 3, y: 22 },
                    Position { x: 4, y: 22 },
                    Position { x: 4, y: 21 },
                    Position { x: 5, y: 21 },
                ]
            }
            Tetromino::J => {
                vec![
                    Position { x: 3, y: 22 },
                    Position { x: 3, y: 21 },
                    Position { x: 4, y: 21 },
                    Position { x: 5, y: 21 },
                ]
            }
            Tetromino::L => {
                vec![
                    Position { x: 3, y: 21 },
                    Position { x: 4, y: 21 },
                    Position { x: 5, y: 21 },
                    Position { x: 5, y: 22 },
                ]
            }
        }
    }

    /// Basic Rotation
    /// The basic rotation states are shown in the diagram on the right. Some points to note:
    ///
    /// When unobstructed, the tetrominoes all appear to rotate purely about a single point. These apparent rotation centers are shown as circles in the diagram.
    /// It is a pure rotation in a mathematical sense.
    /// As a direct consequence, the J, L, S, T and Z tetrominoes have 1 of their 4 states (the spawn state) in a "floating" position where they are not in contact with the bottom of their bounding box.
    /// This allows the bounding box to descend below the surface of the stack (or the floor of the playing field) making it impossible for the tetrominoes to be rotated without the aid of floor kicks.
    /// The S, Z and I tetrominoes have two horizontally oriented states and two vertically oriented states. It can be argued that having two vertical states leads to faster finesse.
    /// For the "I" and "O" tetrominoes, the apparent rotation center is at the intersection of gridlines, whereas for the "J", "L", "S", "T" and "Z" tetrominoes, the rotation center coincides with the center of one of the four constituent minos.
    /// | Guidelines
    pub fn try_basic_rotation(
        &self,
        positions: &[Position],
        current_rotation: &Rotation,
    ) -> Vec<Position> {
        let mut new_positions = positions.to_owned();
        match self {
            Tetromino::I => {
                match current_rotation {
                    Rotation::Zero => {
                        new_positions[0].x += 2;
                        new_positions[0].y += 1;
                        new_positions[1].x += 1;
                        new_positions[2].y -= 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y -= 2;
                    }
                    Rotation::Ninety => {
                        new_positions[0].x += 1;
                        new_positions[0].y -= 2;
                        new_positions[1].y -= 1;
                        new_positions[2].x -= 1;
                        new_positions[3].x -= 2;
                        new_positions[3].y += 1;
                    }
                    Rotation::OneEighty => {
                        new_positions[0].x -= 2;
                        new_positions[0].y -= 1;
                        new_positions[1].x -= 1;
                        new_positions[2].y += 1;
                        new_positions[3].x += 1;
                        new_positions[3].y += 2;
                    }
                    Rotation::TwoHundredSeventy => {
                        new_positions[0].x -= 1;
                        new_positions[0].y += 2;
                        new_positions[1].y += 1;
                        new_positions[2].x += 1;
                        new_positions[3].x += 2;
                        new_positions[3].y -= 1;
                    }
                }
            }
            Tetromino::O => {}
            Tetromino::T => {
                match current_rotation {
                    Rotation::Zero => {
                        new_positions[0].x += 1;
                        new_positions[0].y += 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y -= 1;
                        new_positions[3].x += 1;
                        new_positions[3].y -= 1;
                    }
                    Rotation::Ninety => {
                        new_positions[0].x += 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y += 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y -= 1;
                    }
                    Rotation::OneEighty => {
                        new_positions[0].x -= 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x += 1;
                        new_positions[2].y += 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y += 1;
                    }
                    Rotation::TwoHundredSeventy => {
                        new_positions[0].x -= 1;
                        new_positions[0].y += 1;
                        new_positions[2].x += 1;
                        new_positions[2].y -= 1;
                        new_positions[3].x += 1;
                        new_positions[3].y += 1;
                    }
                }
            }
            Tetromino::S => {
                match current_rotation {
                    Rotation::Zero => {
                        new_positions[0].x += 1;
                        new_positions[0].y += 1;
                        new_positions[2].x += 1;
                        new_positions[2].y -= 1;
                        new_positions[3].y -= 2;
                    }
                    Rotation::Ninety => {
                        new_positions[0].x += 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y -= 1;
                        new_positions[3].x -= 2;
                    }
                    Rotation::OneEighty => {
                        new_positions[0].x -= 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y += 1;
                        new_positions[3].y += 2;
                    }
                    Rotation::TwoHundredSeventy => {
                        new_positions[0].x -= 1;
                        new_positions[0].y += 1;
                        new_positions[2].x += 1;
                        new_positions[2].y += 1;
                        new_positions[3].x += 2;
                    }
                }
            }
            Tetromino::Z => {
                match current_rotation {
                    Rotation::Zero => {
                        new_positions[0].x += 2;
                        new_positions[1].x += 1;
                        new_positions[1].y -= 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y -= 1;
                    }
                    Rotation::Ninety => {
                        new_positions[0].y -= 2;
                        new_positions[1].x -= 1;
                        new_positions[1].y -= 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y += 1;
                    }
                    Rotation::OneEighty => {
                        new_positions[0].x -= 2;
                        new_positions[1].x -= 1;
                        new_positions[1].y += 1;
                        new_positions[3].x += 1;
                        new_positions[3].y += 1;
                    }
                    Rotation::TwoHundredSeventy => {
                        new_positions[0].y += 2;
                        new_positions[1].x += 1;
                        new_positions[1].y += 1;
                        new_positions[3].x += 1;
                        new_positions[3].y -= 1;
                    }
                }
            }
            Tetromino::J => {
                match current_rotation {
                    Rotation::Zero => {
                        new_positions[0].x += 2;
                        new_positions[1].x += 1;
                        new_positions[1].y += 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y -= 1;
                    }
                    Rotation::Ninety => {
                        new_positions[0].y -= 2;
                        new_positions[1].x += 1;
                        new_positions[1].y -= 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y += 1;
                    }
                    Rotation::OneEighty => {
                        new_positions[0].x -= 2;
                        new_positions[1].x -= 1;
                        new_positions[1].y -= 1;
                        new_positions[3].x += 1;
                        new_positions[3].y += 1;
                    }
                    Rotation::TwoHundredSeventy => {
                        new_positions[0].y += 2;
                        new_positions[1].x -= 1;
                        new_positions[1].y += 1;
                        new_positions[3].x += 1;
                        new_positions[3].y -= 1;
                    }
                }
            }
            Tetromino::L => {
                match current_rotation {
                    Rotation::Zero => {
                        new_positions[0].x += 1;
                        new_positions[0].y += 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y -= 1;
                        new_positions[3].y -= 2;
                    }
                    Rotation::Ninety => {
                        new_positions[0].x += 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y += 1;
                        new_positions[3].x -= 2;
                    }
                    Rotation::OneEighty => {
                        new_positions[0].x -= 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x += 1;
                        new_positions[2].y += 1;
                        new_positions[3].y += 2;
                    }
                    Rotation::TwoHundredSeventy => {
                        new_positions[0].x -= 1;
                        new_positions[0].y += 1;
                        new_positions[2].x += 1;
                        new_positions[2].y -= 1;
                        new_positions[3].x += 2;
                    }
                }
            }
        }
        new_positions
    }

    #[allow(dead_code)]
    pub fn as_drawables(&self) -> Vec<Drawable> {
        let mut drawables = Vec::new();
        for position in self.start_positions() {
            let mut data = vec![0.5f32, 0.5f32, 0.5f32, 0.0, self.color()[0], self.color()[1], self.color()[2]];
            data.resize(8, 0.0);
            let d = Drawable::with_shape_data(position.x as isize, position.y as isize, 6, data.try_into().unwrap(), Some(2));
            drawables.push(d);
        }
        drawables
    }
}

#[derive(Component)]
pub struct Tetr {
    pub positions: Vec<Position>,
    pub rotation: Rotation,
    pub tetromino: Tetromino,
}

impl Tetr {
    pub fn new(tetromino: Tetromino) -> Self {
        let positions = tetromino.start_positions();
        Tetr {
            positions,
            rotation: Rotation::Zero,
            tetromino,
        }
    }

    pub fn as_drawables(&self) -> Vec<Drawable> {
        let mut drawables = Vec::new();
        for position in &self.positions {
            let mut data = vec![0.5f32, 0.5f32, 0.5f32, 0.0, self.tetromino.color()[0], self.tetromino.color()[1], self.tetromino.color()[2]];
            data.resize(8, 0.0);
            let d = Drawable::with_shape_data(position.x as isize, position.y as isize, 6, data.try_into().unwrap(), Some(2));
            drawables.push(d);
        }
        drawables
    }

    pub fn offset(&self) -> u64 {
        (std::mem::size_of::<Drawable>() * self.positions.len()) as u64
    }

    pub fn spin(&mut self) {
        self.positions = self.tetromino.try_basic_rotation(self.positions.as_slice(), &self.rotation);
        while self.positions.iter().any(|p| p.x < 0) {
            self.positions.iter_mut().for_each(|p| p.x += 1);
        }
        while self.positions.iter().any(|p| p.x > 9) {
            self.positions.iter_mut().for_each(|p| p.x -= 1);
        }

        self.rotation = match self.rotation {
            Rotation::Zero => Rotation::Ninety,
            Rotation::Ninety => Rotation::OneEighty,
            Rotation::OneEighty => Rotation::TwoHundredSeventy,
            Rotation::TwoHundredSeventy => Rotation::Zero,
        }
    }
}

#[derive(Resource)]
pub struct TetroQueue {
    queue: VecDeque<Tetromino>
}

impl Default for TetroQueue {
    fn default() -> Self {
        TetroQueue {
            queue: VecDeque::new()
        }
    }

}

impl TetroQueue {

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn push(&mut self, tetromino: Tetromino) {
        self.queue.push_back(tetromino);
    }

    pub fn pop(&mut self) -> Option<Tetromino> {
        self.queue.pop_front()
    }

    pub fn get(&self, index: usize) -> Option<&Tetromino> {
        self.queue.get(index)
    }


    pub fn fill_queue(&mut self, mut rng: &mut GlobalRng) {
        let mut bag = vec![Tetromino::I, Tetromino::O, Tetromino::T, Tetromino::S, Tetromino::Z, Tetromino::J, Tetromino::L];
        for _ in 0..7 {
            let index = rng.u8(0..bag.len() as u8) as usize;
            let tetromino = bag.remove(index);
            self.push(tetromino);
        }
    }
}