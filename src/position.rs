use chess_pgn_parser::Move;
use pleco::Board;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::rc::Rc;

use crate::conversion::move_matches_bitmove;
use crate::error::Error;

#[derive(Debug)]
pub struct Position {
    fen: Fen,
    board: Board,
    frequency: f64,
    transitions: HashMap<Fen, Frequency>,
}

#[derive(Debug)]
pub struct Frequency {
    pub frequency: f64,
}

#[derive(Debug, Clone)]
pub struct Fen {
    fen_str: Rc<String>,
    shortened_fen_str: Rc<String>,
}

impl Fen {
    pub fn new(fen_str: &str) -> Self {
        let mut shortened_fen_str = fen_str.to_owned();
        let last_space_index = fen_str.rfind(' ').unwrap();
        let second_to_last_space_index = fen_str[0..last_space_index].rfind(' ').unwrap();
        shortened_fen_str.truncate(second_to_last_space_index);
        Fen {
            fen_str: Rc::new(fen_str.to_owned()),
            shortened_fen_str: Rc::new(shortened_fen_str),
        }
    }

    pub fn starting_board() -> Self {
        Fen::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    pub fn fen_str(&self) -> &str {
        self.fen_str.as_ref()
    }
}

impl std::hash::Hash for Fen {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.shortened_fen_str.hash(state);
    }
}

impl PartialEq for Fen {
    fn eq(&self, other: &Self) -> bool {
        self.shortened_fen_str == other.shortened_fen_str
    }
}
impl Eq for Fen {}

impl Serialize for Fen {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.shortened_fen_str.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Fen {
    fn deserialize<S>(deserializer: S) -> std::result::Result<Self, S::Error>
    where
        S: Deserializer<'de>,
    {
        let shortened_fen_str = Rc::new(String::deserialize(deserializer)?);
        Ok(Self {
            fen_str: shortened_fen_str.clone(),
            shortened_fen_str: shortened_fen_str.clone(),
        })
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pieces: HashMap<char, char> = HashMap::new();
        pieces.insert('K', '♚');
        pieces.insert('Q', '♛');
        pieces.insert('R', '♜');
        pieces.insert('B', '♝');
        pieces.insert('N', '♞');
        pieces.insert('P', '♟');
        pieces.insert('k', '♔');
        pieces.insert('q', '♕');
        pieces.insert('r', '♖');
        pieces.insert('b', '♗');
        pieces.insert('n', '♘');
        pieces.insert('p', '♙');

        let original = self.board.pretty_string();
        let mut pretty = String::with_capacity(original.len());
        for c in original.chars() {
            match pieces.get(&c) {
                Some(rep) => pretty.push(*rep),
                None => pretty.push(c),
            }
        }

        // Remove trailing new line
        pretty.pop();

        // Turn the board over if it's Black to play
        if self.board.turn() == pleco::Player::Black {
            pretty = pretty.chars().rev().collect();
        }

        pretty.push_str(&format!(
            "\nEncountered once in ~{:.0} {} games ({:.6}%)\nYou have prepared {} moves here.\n",
            (1.0 / self.frequency()).round(),
            self.board.turn(),
            100.0 * self.frequency(),
            self.transition_count()
        ));
        if self.transition_count() > 0 {
            pretty.push_str(&format!(
                "Likelihood for any single prepared move to be useful: {:.6}%\n",
                100.0 * self.frequency() / self.transition_count() as f64
            ));
        }
        pretty.fmt(f)
    }
}

impl Position {
    fn illegal_uci_move(&self, uci: &str) -> Error {
        Error::IllegalMove {
            fen_str: self.fen.fen_str().to_owned(),
            mv: uci.to_owned(),
        }
    }

    fn illegal_move(&self, mv: &Move) -> Error {
        Error::IllegalMove {
            fen_str: self.fen.fen_str().to_owned(),
            mv: format!("{:?}", mv),
        }
    }

    fn ambiguous_move(&self, mv: &Move) -> Error {
        Error::AmbiguousMove {
            fen_str: self.fen.fen_str().to_owned(),
            mv: format!("{:?}", mv),
        }
    }

    pub fn apply_move(&mut self, mv: &Move) -> Result<Fen, Error> {
        let mut new_board = self.board.clone();
        let mut candidates = new_board
            .generate_moves()
            .into_iter()
            .filter(|bmv| move_matches_bitmove(mv, *bmv, &self.board));
        let bmv = candidates.next().ok_or_else(|| self.illegal_move(mv))?;
        if candidates.next().is_some() {
            return Err(self.ambiguous_move(mv));
        }
        new_board.apply_move(bmv);
        let new_fen = Fen::new(&new_board.fen());
        self.transitions
            .insert(new_fen.clone(), Frequency { frequency: 0.0 });
        Ok(new_fen)
    }

    pub fn apply_uci(&mut self, uci: &str, frequency: f64) -> Result<Fen, Error> {
        let mut new_board = self.board.clone();
        if !new_board.apply_uci_move(uci) {
            return Err(self.illegal_uci_move(uci));
        }
        let new_fen = Fen::new(&new_board.fen());
        self.transitions
            .insert(new_fen.clone(), Frequency { frequency });
        Ok(new_fen)
    }

    pub fn frequency(&self) -> &f64 {
        &self.frequency
    }

    pub fn fen(&self) -> &Fen {
        &self.fen
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn increase_frequency(&mut self, fdelta: f64) {
        self.frequency += fdelta;
    }

    pub fn transitions(&self) -> impl Iterator<Item = (&Fen, &Frequency)> {
        self.transitions.iter()
    }

    pub fn frequencies_mut(&mut self) -> impl Iterator<Item = &mut Frequency> {
        self.transitions.values_mut()
    }

    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }
}

pub struct PositionCache {
    map: HashMap<Fen, Position>,
}

impl PositionCache {
    pub fn new() -> Self {
        PositionCache {
            map: std::collections::HashMap::new(),
        }
    }

    pub fn position(&mut self, fen: &Fen) -> &mut Position {
        self.map.entry(fen.clone()).or_insert_with(|| Position {
            fen: fen.clone(),
            board: Board::from_fen(&fen.fen_str).unwrap(),
            frequency: 0.0,
            transitions: HashMap::new(),
        })
    }

    pub fn all_positions(&self) -> impl Iterator<Item = &Position> {
        self.map.values()
    }

    pub fn all_positions_mut(&mut self) -> impl Iterator<Item = &mut Position> {
        self.map.values_mut()
    }
}
