pub mod cache;
pub mod lichess;

use crate::position::Fen;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BookMove {
    pub uci: String,
    pub frequency: f64,
}
type BookMoves = Vec<BookMove>;

pub trait OpeningBook {
    fn moves(&mut self, fen: &Fen) -> BookMoves;
}
