use colored::Colorize;
use itertools::Itertools;
use rand::prelude::*;
use rand_distr::{Distribution, Gamma};
use std::collections::HashMap;
use std::f64::consts::E;
use std::fs::File;
use std::io::{self, Write};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

type Piece = Vec<Vec<i32>>;
type State = Vec<(char, usize, i32, i32)>;
type Matrix = Vec<Vec<char>>;

const ALPHA: f64 = 3.0;

fn main() {
    tetrominos::run();
}

fn main2() {
    let mut pieces: HashMap<char, Vec<Piece>> = HashMap::new();

    // Define the tetrominoes pieces with their orientations
    pieces.insert(
        'C',
        vec![
            vec![vec![1, 1], vec![1, 0], vec![1, 1]],
            vec![vec![1, 0, 1], vec![1, 1, 1]],
            vec![vec![1, 1], vec![0, 1], vec![1, 1]],
            vec![vec![1, 1, 1], vec![1, 0, 1]],
        ],
    );

    pieces.insert('O', vec![vec![vec![1, 1], vec![1, 1]]]);

    pieces.insert(
        'J',
        vec![
            vec![vec![1, 0], vec![1, 1]],
            vec![vec![0, 1], vec![1, 1]],
            vec![vec![1, 1], vec![0, 1]],
            vec![vec![1, 1], vec![1, 0]],
        ],
    );

    pieces.insert(
        'I',
        vec![
            vec![vec![1, 1, 1, 1]],
            vec![vec![1], vec![1], vec![1], vec![1]],
        ],
    );

    pieces.insert('B', vec![vec![vec![1, 1]], vec![vec![1], vec![1]]]);

    pieces.insert(
        'T',
        vec![
            vec![vec![0, 1, 0], vec![1, 1, 1]],
            vec![vec![0, 1], vec![1, 1], vec![0, 1]],
            vec![vec![1, 1, 1], vec![0, 1, 0]],
            vec![vec![1, 0], vec![1, 1], vec![1, 0]],
        ],
    );

    pieces.insert(
        'L',
        vec![
            vec![vec![1, 0], vec![1, 0], vec![1, 1]],
            vec![vec![0, 1], vec![0, 1], vec![1, 1]],
            vec![vec![0, 0, 1], vec![1, 1, 1]],
            vec![vec![1, 0, 0], vec![1, 1, 1]],
            vec![vec![1, 1], vec![0, 1], vec![0, 1]],
            vec![vec![1, 1], vec![1, 0], vec![1, 0]],
            vec![vec![1, 1, 1], vec![1, 0, 0]],
            vec![vec![1, 1, 1], vec![0, 0, 1]],
        ],
    );

    pieces.insert(
        'S',
        vec![
            vec![vec![1, 0], vec![1, 1], vec![0, 1]],
            vec![vec![0, 1, 1], vec![1, 1, 0]],
            vec![vec![0, 1], vec![1, 1], vec![1, 0]],
            vec![vec![1, 1, 0], vec![0, 1, 1]],
        ],
    );

    // Initialize simulation parameters
    let mut rng = rand::rng();
    let mu = 25.0;
    let var = 30.0;
    let theta = var / mu;
    let beta = rand_distr::Gamma::new(mu / theta, theta)
        .unwrap()
        .sample(&mut rng);

    let mut p_displace = 0.25;
    let mut p_rotate = 0.25;
    let mut p_add = 0.25;
    let mut p_remove = 0.25;

    let mut state: State = Vec::new();
    let mut best: State = Vec::new();
    let mut step = 0;
    let mut e_min: Option<f64> = None;
    let mut best_step: Option<usize> = None;

    let e0 = energy(&render(&Vec::new(), &pieces), &pieces);

    let now = Instant::now();
    let mut output_file = File::create("BEST.txt").expect("Failed to create output file");

    // 28 seg
    for _ in 0..1_000_000 {
        if step % 500 == 0 {
            state = best.clone();
        }

        let m = render(&state, &pieces);
        let mut e: Option<f64> = None;

        if step % 5000 == 0 {
            let gamma = Gamma::new(mu / theta, theta).unwrap();
            let beta = gamma.sample(&mut rng);

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
            if step % 100 == 0 {
                clear_screen();
            }

            e = Some(energy(&m, &pieces));

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

            if step % 100 == 0 {
                println!(
                    "step {}: E/E0 = {} beta = {} probs = {} {} {} {}",
                    step,
                    e.unwrap() / e0,
                    beta,
                    p_displace,
                    p_rotate,
                    p_add,
                    p_remove
                );
                visualize(&m);
                println!();
                println!("Simulation time {:?}", now.elapsed());
                println!(
                    "E_best/E0 = {} (best since step {})",
                    e_min.unwrap() / e0,
                    best_step.unwrap_or(0)
                );

                let available_pieces = pieces
                    .keys()
                    .map(|&shape| {
                        let count = shape_count(shape, &best);
                        format!("{}", shape.to_string().repeat(5 - count))
                    })
                    .join(" ");

                println!("Available pieces: {}", available_pieces);
            }

            let m_best = render(&best, &pieces);

            if step % 100 == 0 {
                let empty_cells = m_best.iter().flatten().filter(|&&cell| cell == '#').count();

                println!("Empty cells: {}", empty_cells);
                visualize(&m_best);
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

            let new_m = render(&state, &pieces);
            let new_e = energy(&new_m, &pieces);

            if new_e < e.unwrap_or(f64::MAX)
                || (E.ln().ln() as f64) < -beta * (new_e - e.unwrap_or(0.0)) / e0
            {
                // Accept the move
            } else {
                state = state0;
            }
        }

        if pieces.keys().any(|&shape| shape_count(shape, &state) < 5) && rng.random::<f64>() < p_add
        {
            // Proposal to add a piece to the board
            let state0 = state.clone();
            let mut shapes = Vec::new();

            for (&s, _) in &pieces {
                let count = shape_count(s, &state);
                if count < 5 {
                    for _ in 0..(5 - count) {
                        shapes.push(s);
                    }
                }
            }

            if !shapes.is_empty() {
                let shape = *shapes.choose(&mut rng).unwrap();
                let orientation = rng.random_range(0..pieces[&shape].len());
                let x = rng.random_range(0..10);
                let y = rng.random_range(0..15);

                state.push((shape, orientation, x, y));
                let new_m = render(&state, &pieces);
                let new_e = energy(&new_m, &pieces);

                if new_e < e.unwrap_or(f64::MAX)
                    || (E.ln().ln() as f64) < -beta * (new_e - e.unwrap_or(0.0)) / e0
                {
                    // Accept the move
                } else {
                    state = state0;
                }
            }
        }

        if !state.is_empty() && rng.random::<f64>() < p_displace {
            // Proposal to displace a piece on the board
            let state0 = state.clone();
            let k = rng.random_range(0..state.len());
            let dx = rng.random_range(-1..=1);
            let dy = rng.random_range(-1..=1);

            let (shape, orientation, x, y) = state[k];
            state[k] = (shape, orientation, x + dx, y + dy);

            let new_m = render(&state, &pieces);
            let new_e = energy(&new_m, &pieces);

            if new_e < e.unwrap_or(f64::MAX)
                || (E.ln().ln() as f64) < -beta * (new_e - e.unwrap_or(0.0)) / e0
            {
                // Accept the move
            } else {
                state = state0;
            }
        }

        if !state.is_empty() && rng.random::<f64>() < p_rotate {
            // Proposal to rotate a piece on the board
            let state0 = state.clone();
            let k = rng.random_range(0..state.len());
            let (shape, _, x, y) = state[k];
            let new_orientation = rng.random_range(0..pieces[&shape].len());

            state[k] = (shape, new_orientation, x, y);
            let new_m = render(&state, &pieces);
            let new_e = energy(&new_m, &pieces);

            if new_e < e.unwrap_or(f64::MAX)
                || (E.ln().ln() as f64) < -beta * (new_e - e.unwrap_or(0.0)) / e0
            {
                // Accept the move
            } else {
                state = state0;
            }
        }

        step += 1;
    }
}

fn render(state: &State, pieces: &HashMap<char, Vec<Piece>>) -> Matrix {
    let mut m = vec![vec!['#'; 10]; 15];

    for &(shape, orientation, x, y) in state {
        assert!(orientation < pieces[&shape].len());

        for (this_y, row) in pieces[&shape][orientation].iter().enumerate() {
            if !(0 <= (y + this_y as i32) && (y + this_y as i32) < 15) {
                return Vec::new();
            }

            for (this_x, &w) in row.iter().enumerate() {
                if !(0 <= (x + this_x as i32) && (x + this_x as i32) < 10) {
                    return Vec::new();
                }

                let y_idx = (y + this_y as i32) as usize;
                let x_idx = (x + this_x as i32) as usize;

                if m[y_idx][x_idx] != '#' {
                    m[y_idx][x_idx] = 'X';
                } else if w == 1 {
                    m[y_idx][x_idx] = shape;
                }
            }
        }
    }

    m
}

fn visualize(m: &Matrix) {
    println!("----------------------------------");
    for j in 0..15 {
        print!("|");
        for i in 0..10 {
            let cell = m[j][i];
            if cell == '#' {
                print!("   ");
            } else {
                let display = format!(" {} ", cell);
                match cell {
                    'C' => print!("{}", display.green()),
                    'O' => print!("{}", display.yellow()),
                    'J' => print!("{}", display.blue()),
                    'I' => print!("{}", display.cyan()),
                    'B' => print!("{}", display.magenta()),
                    'T' => print!("{}", display.red()),
                    'L' => print!("{}", display.bright_green()),
                    'S' => print!("{}", display.bright_blue()),
                    'X' => print!("{}", display.bright_red()),
                    _ => print!(" {} ", cell),
                }
            }
        }
        println!("  |");
    }
    println!("----------------------------------");
}

fn energy(m: &Matrix, pieces: &HashMap<char, Vec<Piece>>) -> f64 {
    if m.is_empty() {
        return 1.0E+20;
    }

    let mut ee = 0.0;

    for j in 0..15 {
        for i in 0..10 {
            if m[j][i] == 'X' {
                ee += 5.0E+5;
            } else if m[j][i] == '#' {
                for n in 0..15 {
                    for m_idx in 0..10 {
                        if m_idx != i && n != j && m[n][m_idx] == '#' {
                            ee += ((m_idx as f64 - i as f64).powi(2)
                                + (n as f64 - j as f64).powi(2))
                            .powf(0.5 * ALPHA);
                        }
                    }
                }
            }
        }
    }

    0.5 * ee
}

fn shape_count(shape: char, state: &State) -> usize {
    state.iter().filter(|&&(s, _, _, _)| s == shape).count()
}

fn clear_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "cls"]).status().unwrap();
    } else {
        Command::new("clear").status().unwrap();
    }
}
