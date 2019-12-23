use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::{thread, time};

use crate::error::Error;
use crate::opening_book::{BookMove, BookMoves, OpeningBook};
use crate::position::Fen;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Move {
    uci: String,
    san: String,
    white: u32,
    draws: u32,
    black: u32,
    average_rating: u32,
}

#[derive(Deserialize, Debug)]
struct Player {
    name: String,
    rating: u32,
}

#[derive(Deserialize, Debug)]
struct Game {
    id: String,
    winner: String,
    speed: String,
    white: Player,
    black: Player,
    year: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Book {
    white: u32,
    draws: u32,
    black: u32,
    moves: Vec<Move>,
    top_games: Vec<Game>,
}

pub struct Lichess {
    client: Client,
}

static CLOCK_SPEED: &str = "speeds%5B%5D=rapid&speeds%5B%5D=classical&speeds%5B%5D=blitz";
static VARIANT: &str = "variant=standard";
static MOVE_NUMBER: &str = "moves=20";
static RATING: &str = "ratings%5B%5D=2500&ratings%5B%5D=2200&ratings%5B%5D=2000&ratings%5B%5D=1800&ratings%5B%5D=1600";

impl Lichess {
    pub fn new() -> Self {
        Lichess {
            client: Client::new(),
        }
    }

    fn url(&self, fen: &str) -> String {
        let escaped_fen = fen.replace(" ", "%20");
        format!(
            "https://explorer.lichess.ovh/lichess?fen={}&{}&{}&{}&{}",
            escaped_fen, MOVE_NUMBER, VARIANT, CLOCK_SPEED, RATING
        )
    }

    fn get_url(&self, url: &str) -> Result<Book, Error> {
        let mut response = self.client.get(url).send()?;
        match response.status() {
            StatusCode::OK => {
                let lbook: Book = response.json().unwrap();
                Ok(lbook)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                thread::sleep(time::Duration::from_secs(10));
                self.get_url(url)
            }
            code => {
                println!("Error accessing lichess API: HTTP Response Code {}", code);
                Err(Error::Http)
            }
        }
    }

    fn convert_to_pleco_uci(uci: &str, san: &str) -> String {
        if san.starts_with("O-O") {
            uci.replace('a', "c").replace('h', "g")
        } else {
            uci.to_owned()
        }
    }
}

impl OpeningBook for Lichess {
    fn moves(&mut self, fen: &Fen) -> BookMoves {
        /* Here!!! */
        let book = self.get_url(&self.url(&fen.fen_str())).unwrap();
        let total_games = f64::from(book.white + book.draws + book.black);
        book.moves
            .iter()
            .map(|mv| BookMove {
                uci: Lichess::convert_to_pleco_uci(&mv.uci, &mv.san),
                frequency: f64::from(mv.white + mv.draws + mv.black) / total_games,
            })
            .collect()
    }
}
