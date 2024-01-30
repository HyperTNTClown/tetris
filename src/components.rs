use bevy::prelude::*;

#[derive(Resource)]
pub struct BufferUpdate(pub(crate) bool);

#[derive(Component)]
pub struct Updated(pub(crate) bool);

#[derive(Component)]
pub struct Locked;

#[repr(C)]
#[derive(Copy, Clone, Debug, Component)]
pub struct Drawable {
    pub position: [f32; 3],
    pub shape_data: [f32; 8],
    pub shape: u32,
}

impl Drawable {

    pub fn new(x: isize, y: isize, z: isize, shape: Option<u32>) -> Self {
        let shape = shape.unwrap_or(0);
        Drawable {
            position: [x as f32, y as f32, z as f32],
            shape_data: [0.0; 8],
            shape,
            ..default()
        }
    }

    pub fn with_shape_data(x: isize, y: isize, z: isize, shape_data: [f32; 8], shape: Option<u32>) -> Self {
        let shape = shape.unwrap_or(0);
        Drawable {
            position: [x as f32, y as f32, z as f32],
            shape_data,
            shape,
            ..default()
        }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Default for Drawable {
    fn default() -> Self {
        Drawable {
            position: [0.0, 0.0, 0.0],
            shape_data: [0.0; 8],
            shape: 0,
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
enum Rotation {
    ZERO = 0,
    NINETY = 1,
    ONE_EIGHTY = 2,
    TWO_HUNDRED_SEVENTY = 3,
}

#[derive(Debug)]
enum RotationDirection {
    CLOCKWISE = 1,
    COUNTER_CLOCKWISE = -1,
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
    pub score: u32,
    pub level: u32,
}

impl Default for TetrisGame {
    fn default() -> Self {
        TetrisGame {
            field: [[false; 10]; 40],
            next: None,
            hold: None,
            score: 0,
            level: 0,
        }
    }
}

#[derive(Component, Debug)]
pub enum Tetromino {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
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
        positions: &Vec<Position>,
        current_rotation: &Rotation,
        rotation_direction: RotationDirection
    ) -> Option<Vec<Position>> {
        let mut new_positions = positions.clone();
        match self {
            Tetromino::I => {
                match current_rotation {
                    Rotation::ZERO => {
                        new_positions[0].x += 2;
                        new_positions[0].y += 1;
                        new_positions[1].x += 1;
                        new_positions[2].y -= 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y -= 2;
                    }
                    Rotation::NINETY => {
                        new_positions[0].x += 1;
                        new_positions[0].y -= 2;
                        new_positions[1].y -= 1;
                        new_positions[2].x -= 1;
                        new_positions[3].x -= 2;
                        new_positions[3].y += 1;
                    }
                    Rotation::ONE_EIGHTY => {
                        new_positions[0].x -= 2;
                        new_positions[0].y -= 1;
                        new_positions[1].x -= 1;
                        new_positions[2].y += 1;
                        new_positions[3].x += 1;
                        new_positions[3].y += 2;
                    }
                    Rotation::TWO_HUNDRED_SEVENTY => {
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
                    Rotation::ZERO => {
                        new_positions[0].x += 1;
                        new_positions[0].y += 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y -= 1;
                        new_positions[3].x += 1;
                        new_positions[3].y -= 1;
                    }
                    Rotation::NINETY => {
                        new_positions[0].x += 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y += 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y -= 1;
                    }
                    Rotation::ONE_EIGHTY => {
                        new_positions[0].x -= 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x += 1;
                        new_positions[2].y += 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y += 1;
                    }
                    Rotation::TWO_HUNDRED_SEVENTY => {
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
                    Rotation::ZERO => {
                        new_positions[0].x += 1;
                        new_positions[0].y += 1;
                        new_positions[2].x += 1;
                        new_positions[2].y -= 1;
                        new_positions[3].y -= 2;
                    }
                    Rotation::NINETY => {
                        new_positions[0].x += 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y -= 1;
                        new_positions[3].x -= 2;
                    }
                    Rotation::ONE_EIGHTY => {
                        new_positions[0].x -= 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y += 1;
                        new_positions[3].y += 2;
                    }
                    Rotation::TWO_HUNDRED_SEVENTY => {
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
                    Rotation::ZERO => {
                        new_positions[0].x += 2;
                        new_positions[1].x += 1;
                        new_positions[1].y -= 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y -= 1;
                    }
                    Rotation::NINETY => {
                        new_positions[0].y -= 2;
                        new_positions[1].x -= 1;
                        new_positions[1].y -= 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y += 1;
                    }
                    Rotation::ONE_EIGHTY => {
                        new_positions[0].x -= 2;
                        new_positions[1].x -= 1;
                        new_positions[1].y += 1;
                        new_positions[3].x += 1;
                        new_positions[3].y += 1;
                    }
                    Rotation::TWO_HUNDRED_SEVENTY => {
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
                    Rotation::ZERO => {
                        new_positions[0].x += 2;
                        new_positions[1].x += 1;
                        new_positions[1].y += 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y -= 1;
                    }
                    Rotation::NINETY => {
                        new_positions[0].y -= 2;
                        new_positions[1].x += 1;
                        new_positions[1].y -= 1;
                        new_positions[3].x -= 1;
                        new_positions[3].y += 1;
                    }
                    Rotation::ONE_EIGHTY => {
                        new_positions[0].x -= 2;
                        new_positions[1].x -= 1;
                        new_positions[1].y -= 1;
                        new_positions[3].x += 1;
                        new_positions[3].y += 1;
                    }
                    Rotation::TWO_HUNDRED_SEVENTY => {
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
                    Rotation::ZERO => {
                        new_positions[0].x += 1;
                        new_positions[0].y += 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y -= 1;
                        new_positions[3].y -= 2;
                    }
                    Rotation::NINETY => {
                        new_positions[0].x += 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x -= 1;
                        new_positions[2].y += 1;
                        new_positions[3].x -= 2;
                    }
                    Rotation::ONE_EIGHTY => {
                        new_positions[0].x -= 1;
                        new_positions[0].y -= 1;
                        new_positions[2].x += 1;
                        new_positions[2].y += 1;
                        new_positions[3].y += 2;
                    }
                    Rotation::TWO_HUNDRED_SEVENTY => {
                        new_positions[0].x -= 1;
                        new_positions[0].y += 1;
                        new_positions[2].x += 1;
                        new_positions[2].y -= 1;
                        new_positions[3].x += 2;
                    }
                }
            }
        }
        Some(new_positions)
    }

    pub fn as_drawables(&self) -> Vec<Drawable> {
        let mut drawables = Vec::new();
        for position in self.start_positions() {
            let mut data = vec![0.5f32, 0.5f32, 0.5f32, self.color()[0], self.color()[1], self.color()[2]];
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
            rotation: Rotation::ZERO,
            tetromino,
        }
    }

    pub fn as_drawables(&self) -> Vec<Drawable> {
        let mut drawables = Vec::new();
        for (i, position) in self.positions.iter().enumerate() {
            let mut data = vec![0.5f32, 0.5f32, 0.5f32, self.tetromino.color()[0], self.tetromino.color()[1], i as f32 / self.positions.len() as f32];
            data.resize(8, 0.0);
            let d = Drawable::with_shape_data(position.x as isize, position.y as isize, 6, data.try_into().unwrap(), Some(2));
            drawables.push(d);
        }
        drawables
    }

    pub fn offset(&self) -> u64 {
        std::mem::size_of::<Drawable>() as u64 * self.positions.len() as u64
    }

    pub fn spin (&mut self) {
        self.positions = self.tetromino.try_basic_rotation(&self.positions.clone(), &self.rotation, RotationDirection::CLOCKWISE).unwrap();

        self.rotation = match self.rotation {
            Rotation::ZERO => Rotation::NINETY,
            Rotation::NINETY => Rotation::ONE_EIGHTY,
            Rotation::ONE_EIGHTY => Rotation::TWO_HUNDRED_SEVENTY,
            Rotation::TWO_HUNDRED_SEVENTY => Rotation::ZERO,
        }
    }
}