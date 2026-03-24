use colored::{ColoredString, Colorize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Shape {
    C,
    O,
    J,
    I,
    B,
    T,
    L,
    S,
}

const PIECE_C: &[u16] = &[0xC8C0, 0xAE00, 0xC4C0, 0xEA00];
const PIECE_O: &[u16] = &[0xCC00];
const PIECE_J: &[u16] = &[0x8C00, 0x4C00, 0xC400, 0xC800];
const PIECE_I: &[u16] = &[0xF000, 0x8888];
const PIECE_B: &[u16] = &[0x8800, 0xC000];
const PIECE_T: &[u16] = &[0xE400, 0x8C80, 0x4E00, 0x4C40];
const PIECE_L: &[u16] = &[
    0x88C0, 0x44C0, 0x2E00, 0x8E00, 0xC440, 0xC880, 0xE800, 0xE200,
];
const PIECE_S: &[u16] = &[0x8C40, 0x6C00, 0x4C80, 0xC600];

pub const PIECES: &[&[u16]] = &[
    PIECE_C, PIECE_O, PIECE_J, PIECE_I, PIECE_B, PIECE_T, PIECE_L, PIECE_S,
];

const PIECE_C_DIM: &[(usize, usize)] = &[(2, 3), (3, 2), (2, 3), (3, 2)];
const PIECE_O_DIM: &[(usize, usize)] = &[(2, 2)];
const PIECE_J_DIM: &[(usize, usize)] = &[(2, 2), (2, 2), (2, 2), (2, 2)];
const PIECE_I_DIM: &[(usize, usize)] = &[(4, 1), (1, 4)];
const PIECE_B_DIM: &[(usize, usize)] = &[(1, 2), (2, 1)];
const PIECE_T_DIM: &[(usize, usize)] = &[(3, 2), (2, 3), (3, 2), (2, 3)];
const PIECE_L_DIM: &[(usize, usize)] = &[
    (2, 3),
    (2, 3),
    (3, 2),
    (3, 2),
    (2, 3),
    (2, 3),
    (3, 2),
    (3, 2),
];
const PIECE_S_DIM: &[(usize, usize)] = &[(2, 3), (3, 2), (2, 3), (3, 2)];
pub const PIECE_DIMS: &[&[(usize, usize)]] = &[
    PIECE_C_DIM,
    PIECE_O_DIM,
    PIECE_J_DIM,
    PIECE_I_DIM,
    PIECE_B_DIM,
    PIECE_T_DIM,
    PIECE_L_DIM,
    PIECE_S_DIM,
];

impl Shape {
    pub const ALL: [Shape; 8] = [
        Shape::C,
        Shape::O,
        Shape::J,
        Shape::I,
        Shape::B,
        Shape::T,
        Shape::L,
        Shape::S,
    ];

    #[inline(always)]
    pub fn as_char(self) -> char {
        match self {
            Shape::C => 'C',
            Shape::O => 'O',
            Shape::J => 'J',
            Shape::I => 'I',
            Shape::B => 'B',
            Shape::T => 'T',
            Shape::L => 'L',
            Shape::S => 'S',
        }
    }

    pub fn as_colored(self) -> ColoredString {
        match self {
            Shape::C => " C ".green(),
            Shape::O => " O ".yellow(),
            Shape::J => " J ".blue(),
            Shape::I => " I ".cyan(),
            Shape::B => " B ".magenta(),
            Shape::T => " T ".red(),
            Shape::L => " L ".bright_green(),
            Shape::S => " S ".bright_blue(),
        }
    }

    #[inline(always)]
    pub fn get_rotations(self) -> &'static [u16] {
        PIECES[self as usize]
    }

    #[inline(always)]
    pub fn get_dims(self) -> &'static [(usize, usize)] {
        PIECE_DIMS[self as usize]
    }

    #[inline(always)]
    pub fn get_dim(self, rot: usize) -> (usize, usize) {
        PIECE_DIMS[self as usize][rot]
    }
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub shape: Shape,
    pub rotation: usize,
    pub x: usize,
    pub y: usize,
}

impl Piece {
    pub fn new(shape: Shape, rot: usize, x: usize, y: usize) -> Self {
        Piece {
            shape,
            rotation: rot,
            x,
            y,
        }
    }
}
