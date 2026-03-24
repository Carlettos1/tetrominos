use colored::{ColoredString, Colorize};

use crate::{
    ALPHA,
    piece::{Piece, Shape},
};

pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Escaque {
    Invalid,
    Empty,
    Piece(Shape),
}

#[derive(Debug, Clone)]
pub struct Matrix {
    pub matrix: [[Escaque; WIDTH]; HEIGHT],
    pub dirty_marks: usize,
    pub bits: [u16; HEIGHT],
}

impl Escaque {
    pub fn as_char(&self) -> char {
        match self {
            Escaque::Empty => '#',
            Escaque::Invalid => 'X',
            Escaque::Piece(shape) => shape.as_char(),
        }
    }

    pub fn as_colored(&self) -> ColoredString {
        match self {
            Escaque::Empty => "   ".into(),
            Escaque::Invalid => " X ".bright_red(),
            Escaque::Piece(shape) => shape.as_colored(),
        }
    }
}

impl Matrix {
    pub fn new(state: &Vec<Piece>) -> Self {
        let mut matrix = [[Escaque::Empty; WIDTH]; HEIGHT];
        let mut bits = [0_u16; HEIGHT];
        let mut dirty_marks = 0;
        for piece in state {
            assert!(piece.rotation < piece.shape.get_rotations().len());
            let placed_piece = piece.shape.get_rotations()[piece.rotation];
            // las piezas son de a lo más de 4x4
            println!("{piece:?}");
            for dy in 0..piece.shape.get_dim(piece.rotation).1 {
                // obtiene el iésimo número (e.g. 0x1234 da 4 para i=0, 3 para i=1, etc)
                let piece_row = ((placed_piece >> (dy * 4)) & 0xF) << piece.x;
                if (bits[piece.y + dy]) & piece_row != 0 {
                    dirty_marks += 1;
                    for col in 0..HEIGHT {
                        if (piece_row & (1 << col)) >> col == 1 {
                            matrix[piece.y + dy][col] = Escaque::Invalid;
                        }
                    }
                } else {
                    // no colission, just draw onto matrix
                    for col in 0..HEIGHT {
                        if (piece_row & (1 << col)) >> col == 1 {
                            matrix[piece.y + dy][col] = Escaque::Piece(piece.shape);
                        }
                    }
                }
                bits[piece.y + dy] |= piece_row;
            }
        }
        Self {
            matrix,
            bits,
            dirty_marks,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|r| r == &0)
    }

    pub fn occupied_cells(&self) -> u32 {
        self.bits.iter().map(|r| r.count_ones()).sum()
    }

    pub fn empty_cells(&self) -> u32 {
        (WIDTH * HEIGHT) as u32 - self.occupied_cells()
    }

    pub fn energy(&self) -> f64 {
        if self.is_empty() {
            return 1.0E+20;
        }

        let mut ee = 0.0;

        for j in 0..15 {
            for i in 0..10 {
                match self.matrix[j][i] {
                    Escaque::Invalid => ee += 5.0E+5,
                    Escaque::Empty => (),
                    Escaque::Piece(_) => {
                        for other_j in 0..15 {
                            for other_i in 0..10 {
                                if i != other_i
                                    && j != other_j
                                    && matches!(self.matrix[other_j][other_i], Escaque::Piece(_))
                                {
                                    ee += ((other_i as f64 - i as f64).powi(2)
                                        + (other_j as f64 - j as f64).powi(2))
                                    .powf(0.5 * ALPHA);
                                }
                            }
                        }
                    }
                }
            }
        }

        0.5 * ee
    }

    pub fn draw(&self) {
        println!("----------------------------------");
        for row in self.matrix.iter() {
            print!("|");
            for e in row.iter() {
                print!("{}", e.as_colored())
            }
            println!("  |")
        }
        println!("----------------------------------");
    }
}

pub fn shape_count(state: &Vec<Piece>, shape: Shape) -> usize {
    state.iter().filter(|piece| piece.shape == shape).count()
}
