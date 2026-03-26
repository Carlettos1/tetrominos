pub mod board;
pub mod piece;

use std::{f64, fs::File, io::Write, process::Command, time::Instant};

use itertools::Itertools;
use rand::{Rng, seq::IndexedRandom};
use rand_distr::{Distribution, Gamma};

use crate::{
    board::{HEIGHT, Matrix, WIDTH, shape_count},
    piece::{PIECES, Piece, Shape},
};

const ALPHA: f64 = 3.0;
/// how many steps needs to happen before a rollback (returning to the best previous state) occurs.
const ROLLBACK_STEPS: usize = 501;

/// how many steps needed to show the current and best state in console
const N_TO_SHOW: usize = 10_000;

/// how many steps needed for probabilities and beta to change
const N_SCRAMBLE: usize = 5000;

pub fn run() {
    // Initialize simulation parameters
    let mut rng = rand::rng();
    let mu = 25.0;
    let var = 30.0;
    let theta = var / mu;
    let mut beta = rand_distr::Gamma::new(mu / theta, theta)
        .unwrap()
        .sample(&mut rng);

    let mut p_displace = 0.25;
    let mut p_rotate = 0.25;
    let mut p_add = 0.25;
    let mut p_remove = 0.25;

    let mut state: Vec<Piece> = Vec::new();
    let mut best: Vec<Piece> = Vec::new();
    let mut step = 0;
    let mut e_min: Option<f64> = None;
    let mut best_step: Option<usize> = None;

    let matrix = Matrix::new(&state);
    let e0 = matrix.energy();

    let now = Instant::now();
    let mut output_file = File::create("BEST.txt").expect("Failed to create output file");
    let mut log_file = File::create("log.log").expect("Failed to create log file");

    for _ in 0..1_000_000 {
        if step % ROLLBACK_STEPS == 0 {
            state = best.clone();
        }

        let m = Matrix::new(&state);
        let mut e: Option<f64> = None;

        if step % N_SCRAMBLE == 0 {
            let gamma = Gamma::new(mu / theta, theta).unwrap();
            beta = gamma.sample(&mut rng);

            p_displace = rng.random::<f64>();
            p_rotate = rng.random::<f64>();
            p_add = rng.random::<f64>();
            p_remove = rng.random::<f64>();

            let sum = p_displace + p_rotate + p_add + p_remove;
            p_displace /= sum;
            p_rotate /= sum;
            p_add /= sum;
            p_remove /= sum;
        }

        if !m.is_empty() {
            e = Some(m.energy());

            if e_min.is_none() || e.unwrap() < e_min.unwrap() {
                e_min = e;
                best = state.clone();

                // Write best state to file
                for item in &best {
                    write!(output_file, "{:?} ", item).unwrap();
                }
                writeln!(output_file).unwrap();
                output_file.flush().unwrap();

                best_step = Some(step);
            }

            if step % N_TO_SHOW == 0 {
                clear_screen();
                println!(
                    "step {}: E/E0 = {:.2} beta = {:.2} probs = {:.2} {:.2} {:.2} {:.2}",
                    step,
                    e.unwrap() / e0,
                    beta,
                    p_displace,
                    p_rotate,
                    p_add,
                    p_remove
                );
                m.draw();
                println!();
                println!("Simulation time {:?}", now.elapsed());
                println!(
                    "E_best/E0 = {} (best since step {})",
                    e_min.unwrap() / e0,
                    best_step.unwrap_or(0)
                );

                let available_pieces = Shape::ALL
                    .iter()
                    .map(|&shape| {
                        let count = shape_count(&best, shape);
                        format!("{}", shape.as_char().to_string().repeat(5 - count))
                    })
                    .join(" ");

                println!("Available pieces: {}", available_pieces);
            }

            let m_best = Matrix::new(&best);

            if step % N_TO_SHOW == 0 {
                let empty_cells = m_best.empty_cells();

                println!("Empty cells: {}", empty_cells);
                m_best.draw();
            }
        }

        // Monte Carlo proposals
        if !state.is_empty() && rng.random::<f64>() < p_remove {
            // Proposal to remove a piece from the board
            let state0 = state.clone();
            let k = rng.random_range(0..state0.len());
            state = state0
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != k)
                .map(|(_, v)| v.clone())
                .collect();

            let new_m = Matrix::new(&state);
            let new_e = new_m.energy();

            if new_e < e.unwrap_or(f64::MAX)
                || (f64::consts::E.ln().ln() as f64) < -beta * (new_e - e.unwrap_or(0.0)) / e0
            {
                // Accept the move
                writeln!(&mut log_file, "Removed {k}").expect("Couldn't write to log file?");
            } else {
                state = state0;
            }
        }

        if Shape::ALL
            .iter()
            .any(|shape| shape_count(&state, *shape) < 5)
            && rng.random::<f64>() < p_add
        {
            // Proposal to add a piece to the board
            let state0 = state.clone();
            let mut shapes = Vec::new();

            for shape in Shape::ALL {
                let count = shape_count(&state, shape);
                if count < 5 {
                    for _ in 0..(5 - count) {
                        shapes.push(shape);
                    }
                }
            }

            if !shapes.is_empty() {
                let shape = *shapes.choose(&mut rng).unwrap();
                let orientation = rng.random_range(0..shape.get_rotations().len());
                let x = rng.random_range(0..(WIDTH - shape.get_dim(orientation).0 + 1));
                let y = rng.random_range(0..(HEIGHT - shape.get_dim(orientation).1 + 1));

                state.push(Piece::new(shape, orientation, x, y));
                let new_m = Matrix::new(&state);
                let new_e = new_m.energy();

                if new_e < e.unwrap_or(f64::MAX)
                    || (f64::consts::E.ln().ln() as f64) < -beta * (new_e - e.unwrap_or(0.0)) / e0
                {
                    // Accept the move
                    writeln!(&mut log_file, "Added {} at {:?}", shape.as_char(), (x, y))
                        .expect("Couldn't write to log file?");
                } else {
                    state = state0;
                }
            }
        }

        if !state.is_empty() && rng.random::<f64>() < p_displace {
            // Proposal to displace a piece on the board
            let k = rng.random_range(0..state.len());
            let dx = rng.random_range(-1..=1_i32);
            let dy = rng.random_range(-1..=1_i32);

            let Piece {
                shape,
                rotation: orientation,
                x,
                y,
            } = state[k];
            let new_x = x.checked_add_signed(dx as isize).unwrap_or_default();
            let new_y = y.checked_add_signed(dy as isize).unwrap_or_default();
            // first check if the new coordinates are legal
            if new_x >= WIDTH
                || new_y >= HEIGHT
                || new_x + shape.get_dim(orientation).0 - 1 >= WIDTH
                || new_y + shape.get_dim(orientation).1 - 1 >= HEIGHT
            {
                // do nothing
            } else {
                let state0 = state.clone();
                state[k] = Piece::new(shape, orientation, new_x, new_y);

                let new_m = Matrix::new(&state);
                let new_e = new_m.energy();

                if new_e < e.unwrap_or(f64::MAX)
                    || (f64::consts::E.ln().ln() as f64) < -beta * (new_e - e.unwrap_or(0.0)) / e0
                {
                    // Accept the move
                    writeln!(&mut log_file, "Displac {k}").expect("Couldn't write to log file?");
                } else {
                    state = state0;
                }
            }
        }

        if !state.is_empty() && rng.random::<f64>() < p_rotate {
            // Proposal to rotate a piece on the board
            let k = rng.random_range(0..state.len());
            let Piece {
                shape,
                rotation: _,
                x,
                y,
            } = state[k];
            let new_orientation = rng.random_range(0..shape.get_rotations().len());

            // again, check if rotation makes the piece position ilegal
            // no need to check if pos is right, as it doesn't change
            if x + shape.get_dim(new_orientation).0 - 1 >= WIDTH
                || y + shape.get_dim(new_orientation).1 - 1 >= HEIGHT
            {
                // do nothing
            } else {
                let state0 = state.clone();
                state[k] = Piece::new(shape, new_orientation, x, y);
                let new_m = Matrix::new(&state);
                let new_e = new_m.energy();

                if new_e < e.unwrap_or(f64::MAX)
                    || (f64::consts::E.ln().ln() as f64) < -beta * (new_e - e.unwrap_or(0.0)) / e0
                {
                    // Accept the move
                    writeln!(&mut log_file, "Rotate  {k}").expect("Couldn't write to log file?");
                } else {
                    state = state0;
                }
            }
        }

        step += 1;
    }
}

fn clear_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "cls"]).status().unwrap();
    } else {
        Command::new("clear").status().unwrap();
    }
}
