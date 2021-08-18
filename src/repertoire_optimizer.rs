use pleco::Player;
use std::fs;
use std::path::PathBuf;

use crate::error::Error;
use crate::opening_book::OpeningBook;
use crate::position::{Fen, Position, PositionCache, AnyMove};

pub struct RepertoireOptimizer {
    me: Player,
    tree: PositionCache,

    pub average_book_length: f64,
}

struct FrequencyDelta {
    fen: Fen,
    fdelta: f64,
    ply: usize,
    sequence: Vec<AnyMove>,
}

impl RepertoireOptimizer {
    pub fn new(me: Player) -> Self {
        RepertoireOptimizer {
            me,
            tree: PositionCache::new(),
            average_book_length: 0.0,
        }
    }

    pub fn read_games(filename: &PathBuf) -> Result<Vec<chess_pgn_parser::Game>, Error> {
        let contents = fs::read_to_string(filename)?;
        Ok(chess_pgn_parser::read_games(&contents).map_err(|_| Error::PgnParser)?)
    }

    pub fn add_game_to_repertoire(&mut self, game: chess_pgn_parser::Game) -> Result<(), Error> {
        let mut fen = Fen::starting_board();
        let mut pos = self.tree.position(&fen);
        let mut sequence = Vec::<AnyMove>::new();
        for mv in game.moves {
            sequence.push(AnyMove::ModelMove(mv.move_.move_.clone()));
            fen = pos.apply_move(&mv.move_.move_)?;
            pos = self.tree.position_w_sequence(&fen, sequence.clone());
        }
        Ok(())
    }

    pub fn add_opponents_moves_from_book(
        &mut self,
        book: &mut dyn OpeningBook,
    ) -> Result<(), Error> {
        let me = self.me;
        let fens: Vec<Result<Fen, Error>> = self
            .tree
            .all_positions_mut()
            .filter(|pos| pos.board().turn() != me)
            .flat_map(|pos| {
                book.moves(pos.fen())
                    .into_iter()
                    .map(move |book_move| {
                        pos.apply_uci(&book_move.uci, &book_move.frequency)
                    })
            })
            .collect::<Vec<_>>();
        for fen in fens {
            self.tree.position(&fen?);
        }
        Ok(())
    }

    pub fn set_own_move_frequencies(&mut self) {
        let me = self.me;
        for position in self
            .tree
            .all_positions_mut()
            .filter(|pos| pos.board().turn() == me && pos.transition_count() > 0)
        {
            let proportional_frequency = 1.0 / position.transition_count() as f64;
            for mut frequency in position.frequencies_mut() {
                frequency.frequency = proportional_frequency;
            }
        }
    }

    pub fn update_position_frequencies(&mut self) {
        let mut positions_to_update = Vec::<FrequencyDelta>::new();
        positions_to_update.push(FrequencyDelta {
            fen: Fen::starting_board(),
            fdelta: 1.0,
            ply: 0,
            sequence: Vec::new(),
        });

        while let Some(FrequencyDelta { fen, fdelta, ply, sequence }) = positions_to_update.pop() {
            if fdelta == 0.0 {
                continue;
            }
            let position = self.tree.position(&fen);
            if position.board().turn() == self.me && position.transition_count() == 0 {
                self.average_book_length += (ply / 2) as f64 * fdelta;
            }
            position.increase_frequency(fdelta);
            position.set_sequence(sequence.clone());
            for (to_fen, transition) in position.transitions() {
                let mut new_sequence = sequence.clone();
                new_sequence.push(transition.mv.clone());
                positions_to_update.push(FrequencyDelta {
                    fen: to_fen.clone(),
                    fdelta: fdelta * transition.frequency,
                    ply: ply + 1,
                    sequence: new_sequence,
                });
            }
        }
    }

    pub fn own_positions(&self) -> Vec<&Position> {
        self.tree
            .all_positions()
            .filter(|pos| pos.board().turn() == self.me)
            .collect()
    }

    pub fn recommend_for_addition<'a>(
        positions: &[&'a Position],
        count: usize,
    ) -> Vec<&'a Position> {
        let mut recommendations = positions.to_owned();
        recommendations.retain(|pos| pos.transition_count() == 0);
        recommendations.sort_by(|a, b| b.frequency().partial_cmp(a.frequency()).unwrap());
        recommendations.truncate(count);
        recommendations
    }

    pub fn recommend_for_removal<'a>(
        positions: &[&'a Position],
        count: usize,
    ) -> Vec<&'a Position> {
        let mut recommendations = positions.to_owned();
        recommendations.retain(|pos| pos.transition_count() > 0);
        recommendations.sort_by(|a, b| a.frequency().partial_cmp(b.frequency()).unwrap());
        recommendations.truncate(count);
        recommendations
    }

    pub fn recommend_for_narrowing<'a>(
        positions: &[&'a Position],
        count: usize,
    ) -> Vec<&'a Position> {
        let mut recommendations = positions.to_owned();
        recommendations.retain(|pos| pos.transition_count() > 1);
        recommendations.sort_by(|a, b| {
            (a.frequency() / a.transition_count() as f64)
                .partial_cmp(&(b.frequency() / b.transition_count() as f64))
                .unwrap()
        });
        recommendations.truncate(count);
        recommendations
    }

    pub fn recommend_for_reduction<'a>(
        positions: &[&'a Position],
        count: usize,
    ) -> Vec<&'a Position> {
        let mut recommendations = positions.to_owned();
        recommendations.retain(|pos| pos.transition_count() > 1);
        recommendations.sort_by(|a, b| {
            (b.frequency() * b.transition_count() as f64)
                .partial_cmp(&(a.frequency() * a.transition_count() as f64))
                .unwrap()
        });
        recommendations.truncate(count);
        recommendations
    }
}
