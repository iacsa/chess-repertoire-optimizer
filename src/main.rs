mod conversion; // Adapter tools between crates chess_pgn_parser and pleco
mod error;
mod opening_book;
mod position;
mod repertoire_optimizer;

use crate::error::Error;
use crate::opening_book::cache::Cache;
use crate::opening_book::lichess::Lichess;
use crate::repertoire_optimizer::RepertoireOptimizer;

use pleco::Player;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;

/// Cover the most ground with the least amount of lines prepared!
#[derive(StructOpt, Debug)]
#[structopt(name = "Chess Repertoire Optimizer")]
struct Opt {
    /// PGN files containing your White repertoire
    #[structopt(short, long, parse(from_os_str))]
    white_repertoire: Vec<PathBuf>,

    /// PGN files containing your Black repertoire
    #[structopt(short, long, parse(from_os_str))]
    black_repertoire: Vec<PathBuf>,

    /// Local file for caching opening book moves
    #[structopt(short, long, parse(from_os_str))]
    cache_file: Option<PathBuf>,

    /// How many frequent positions to recommend for addition
    #[structopt(long, default_value = "10")]
    best: usize,

    /// How many infrequent positions to recommend for removal
    #[structopt(long, default_value = "0")]
    worst: usize,

    /// How many positions with many candidates to show
    #[structopt(long, default_value = "0")]
    most: usize,
}

fn resolve_to_files(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            println!(
                "'{}' is a directory; Importing all files from within...",
                path.display()
            );
            let subpaths = path
                .read_dir()
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
            files.append(&mut resolve_to_files(subpaths));
        } else {
            files.push(path);
        }
    }
    files
}

pub fn main() -> Result<(), Error> {
    let now = Instant::now();

    let opt = Opt::from_args();
    let mut positions = Vec::new();

    let mut white_repertoire_optimizer = RepertoireOptimizer::new(Player::White);
    let mut black_repertoire_optimizer = RepertoireOptimizer::new(Player::Black);
    let mut opening_book = Cache::new(Lichess::new());

    if let Some(ref path) = opt.cache_file {
        if path.exists() {
            match opening_book.load(File::open(path)?) {
                Err(e) => {
                    println!("Failed to read cache file '{}': {:?}", path.display(), e);
                    return Err(e);
                }
                Ok(_) => println!("Cache file '{}' loaded successfully...", path.display()),
            }
        } else {
            println!(
                "Cache file '{}' not found; Will be created...",
                path.display()
            );
        }
    }

    println!("Importing lines...");
    for path in resolve_to_files(opt.white_repertoire) {
        match RepertoireOptimizer::read_games(&path) {
            Ok(games) => {
                println!(
                    "Import of '{}': Found {} games",
                    path.display(),
                    games.len()
                );
                for game in games {
                    if let Err(e) = white_repertoire_optimizer.add_game_to_repertoire(game) {
                        println!("WARNING: '{}' contains bad move: {}", path.display(), e);
                    }
                }
            }
            Err(_) => {
                println!("WARNING: Import of '{}' failed", path.display());
            }
        }
    }
    for path in resolve_to_files(opt.black_repertoire) {
        match RepertoireOptimizer::read_games(&path) {
            Ok(games) => {
                println!(
                    "Import of '{}': Found {} games",
                    path.display(),
                    games.len()
                );
                for game in games {
                    if let Err(e) = black_repertoire_optimizer.add_game_to_repertoire(game) {
                        println!("WARNING: '{}' contains bad move: {}", path.display(), e);
                    }
                }
            }
            Err(_) => {
                println!("WARNING: Import of '{}' failed", path.display());
            }
        }
    }

    println!("checking book moves...");
    white_repertoire_optimizer.add_opponents_moves_from_book(&mut opening_book)?;
    black_repertoire_optimizer.add_opponents_moves_from_book(&mut opening_book)?;
    println!("setting own move frequencies...");
    white_repertoire_optimizer.set_own_move_frequencies();
    black_repertoire_optimizer.set_own_move_frequencies();
    println!("updating position frequencies...");
    white_repertoire_optimizer.update_position_frequencies();
    black_repertoire_optimizer.update_position_frequencies();

    let average_book_length = (white_repertoire_optimizer.average_book_length
        + black_repertoire_optimizer.average_book_length)
        / 2.0;

    positions.append(&mut white_repertoire_optimizer.own_positions());
    positions.append(&mut black_repertoire_optimizer.own_positions());

    println!();
    println!("## Repertoire Statistics ##");
    println!(
        "Average moves you stay in book per game: {} (higher is better)",
        average_book_length
    );
    println!(
        "Your repertoire spans {} positions (lower is better)",
        positions
            .iter()
            .filter(|pos| pos.transition_count() > 0)
            .count()
    );

    if opt.best > 0 {
        println!();
        println!("## Positions you are most likely to encounter where you are out-of-book ##");
        println!("Consider adding these to your repertoire, as it will improve it the most");
        println!();
        for position in RepertoireOptimizer::recommend_for_addition(&positions, opt.best) {
            println!("{}", position);
        }
    }

    if opt.worst > 0 {
        println!();
        println!(
            "## Positions you are least likely to encounter where you have a line prepared ##"
        );
        println!("Consider removing these from your repertoire, as it will have the least impact");
        println!();
        for position in RepertoireOptimizer::recommend_for_removal(&positions, opt.worst) {
            println!("{}", position);
        }
    }

    if opt.most > 0 {
        println!();
        println!("## Positions where your prepared moves are least likely to be used ##");
        println!("Consider reducing the number of different moves you play here");
        println!();
        for position in RepertoireOptimizer::recommend_for_narrowing(&positions, opt.most) {
            println!("{}", position);
        }
    }

    if let Some(ref path) = opt.cache_file {
        if opening_book.has_changed() {
            opening_book.save(File::create(path)?)?;
        }
    }

    println!(
        "Total runtime: {:.2} s",
        now.elapsed().as_millis() as f64 / 1000.0
    );

    Ok(())
}
