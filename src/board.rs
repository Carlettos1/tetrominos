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
            let rotated_shape = piece.shape.get_rotations()[piece.rotation];
            // la vida comienza de nuevo
            // pos x of piece
            let x = piece.x;
            // pos y of piece
            let y = piece.y;
            // dim x of piece
            let a = piece.shape.get_dim(piece.rotation).0;
            // dim y of piece
            let b = piece.shape.get_dim(piece.rotation).1;
            for row in 0..b {
                // S_i
                let shape_row = (rotated_shape >> ((4 - 1 - row) * 4)) & 0xF;
                // PS_i
                let placed_shape_row = shape_row << (12 - x);
                let row_collisions = bits[y + row] & placed_shape_row;
                let row_collisions_number = row_collisions.count_ones();
                if row_collisions_number != 0 {
                    dirty_marks += row_collisions_number as usize;
                    for j in x..(x + a) {
                        if ((row_collisions >> (16 - 1 - j)) & 1) != 0 {
                            matrix[y + row][j] = Escaque::Invalid;
                        } else if ((placed_shape_row >> (16 - 1 - j)) & 1) != 0 {
                            matrix[y + row][j] = Escaque::Piece(piece.shape);
                        }
                    }
                } else {
                    for j in x..(x + a) {
                        if ((placed_shape_row >> (16 - 1 - j)) & 1) != 0 {
                            matrix[y + row][j] = Escaque::Piece(piece.shape);
                        }
                    }
                }
                bits[y + row] |= placed_shape_row;
            }
            // las piezas son de a lo más de 4x4
            #[cfg(debug_assertions)]
            println!(
                "{piece:?} \t<> {:?}",
                (
                    piece.x,
                    piece.y,
                    piece.x + piece.shape.get_dim(piece.rotation).0,
                    piece.y + piece.shape.get_dim(piece.rotation).1
                )
            );
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

        let mut energy = 0.0;

        for j in 0..HEIGHT {
            for i in 0..WIDTH {
                match self.matrix[j][i] {
                    Escaque::Invalid => energy += 5.0E+5,
                    Escaque::Piece(_) => (),
                    Escaque::Empty => {
                        for other_j in 0..15 {
                            for other_i in 0..10 {
                                if i != other_i
                                    && j != other_j
                                    && matches!(self.matrix[other_j][other_i], Escaque::Empty)
                                {
                                    energy += ((other_i.abs_diff(i).pow(2)
                                        + other_j.abs_diff(j).pow(2))
                                        as f64)
                                        .powf(0.5 * ALPHA);
                                }
                            }
                        }
                    }
                }
            }
        }

        0.5 * energy
    }

    pub fn draw(&self) {
        println!("+--------------------------------+");
        for row in self.matrix.iter() {
            print!("|");
            for e in row.iter() {
                print!("{}", e.as_colored())
            }
            println!("  |")
        }
        println!("+--------------------------------+");
        // for row in self.bits.iter() {
        //     println!("|        {row:016b}        |");
        // }
        // println!("+--------------------------------+");
        // println!("Dirty marks: {}", self.dirty_marks);
    }
}

pub fn shape_count(state: &Vec<Piece>, shape: Shape) -> usize {
    state.iter().filter(|piece| piece.shape == shape).count()
}
