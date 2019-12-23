use crate::error::Error;
use crate::opening_book::*;
use crate::position::Fen;

use std::collections::HashMap;
use std::io::{Read, Write};

pub struct Cache<'a> {
    cache: HashMap<Fen, BookMoves>,
    has_changed: bool,
    opening_book: Box<dyn OpeningBook + 'a>,
}

impl<'a> Cache<'a> {
    pub fn new<T: OpeningBook + 'a>(opening_book: T) -> Self {
        Cache {
            cache: HashMap::new(),
            has_changed: false,
            opening_book: Box::new(opening_book),
        }
    }

    pub fn load<T: Read>(&mut self, mut source: T) -> Result<(), Error> {
        let mut data = Vec::new();
        source.read_to_end(&mut data)?;
        self.cache = bincode::deserialize(&data)?;
        self.has_changed = false;
        Ok(())
    }

    pub fn save<T: Write>(&mut self, mut destination: T) -> Result<(), Error> {
        let data = bincode::serialize(&self.cache)?;
        destination.write_all(&data)?;
        self.has_changed = false;
        Ok(())
    }

    pub fn has_changed(&self) -> bool {
        self.has_changed
    }
}

impl OpeningBook for Cache<'_> {
    fn moves(&mut self, fen: &Fen) -> BookMoves {
        let has_changed = &mut self.has_changed;
        let cache = &mut self.cache;
        let opening_book = &mut self.opening_book;
        cache
            .entry(fen.clone())
            .or_insert_with(|| {
                *has_changed = true;
                opening_book.moves(fen)
            })
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::opening_book::*;

    use std::collections::HashMap;

    struct BookDouble {
        configuration: HashMap<Fen, BookMoves>,
    }

    impl BookDouble {
        fn new() -> Self {
            Self {
                configuration: HashMap::new(),
            }
        }

        fn configure(&mut self, fen: Fen, book_moves: BookMoves) -> (Fen, BookMoves) {
            self.configuration.insert(fen.clone(), book_moves.clone());
            (fen, book_moves)
        }
    }

    impl OpeningBook for BookDouble {
        fn moves(&mut self, fen: &Fen) -> BookMoves {
            // Configurations are single-use only
            // This makes sure that the Book is only called once for each Fen
            // => Caching works correctly
            self.configuration.remove(fen).unwrap()
        }
    }

    #[test]
    fn it_passes_book_moves_from_the_internal_book() {
        let mut book = BookDouble::new();
        let (fen_1, book_moves_1) = book.configure(
            Fen::starting_board(),
            vec![BookMove {
                uci: "e2e4".to_owned(),
                frequency: 0.5,
            }],
        );
        let (fen_2, book_moves_2) = book.configure(
            Fen::new("a b c d e f"),
            vec![BookMove {
                uci: "d2d4".to_owned(),
                frequency: 0.3,
            }],
        );
        let (fen_3, book_moves_3) = book.configure(
            Fen::new("a b c D e f"),
            vec![BookMove {
                uci: "c2c4".to_owned(),
                frequency: 0.1,
            }],
        );
        let mut cache = crate::opening_book::cache::Cache::new(book);
        let result_1 = cache.moves(&fen_1);
        let result_2 = cache.moves(&fen_2);
        let result_3 = cache.moves(&fen_3);
        assert_eq!(result_1, book_moves_1);
        assert_eq!(result_2, book_moves_2);
        assert_eq!(result_3, book_moves_3);
    }

    #[test]
    fn it_caches_the_results() {
        let mut book = BookDouble::new();
        let (fen_1, book_moves_1) = book.configure(
            Fen::starting_board(),
            vec![BookMove {
                uci: "e2e4".to_owned(),
                frequency: 0.5,
            }],
        );
        let (fen_2, book_moves_2) = book.configure(
            Fen::new("a b c d e f"),
            vec![BookMove {
                uci: "d2d4".to_owned(),
                frequency: 0.3,
            }],
        );
        let (fen_3, book_moves_3) = book.configure(
            Fen::new("a b c D e f"),
            vec![BookMove {
                uci: "c2c4".to_owned(),
                frequency: 0.1,
            }],
        );
        let mut cache = crate::opening_book::cache::Cache::new(book);

        // Make some requests to induce caching
        let _ = cache.moves(&fen_1);
        let _ = cache.moves(&fen_2);
        let _ = cache.moves(&fen_3);

        // Repeat requests in different order
        let result_2 = cache.moves(&fen_2);
        let result_3 = cache.moves(&fen_3);
        let result_1 = cache.moves(&fen_1);

        assert_eq!(result_1, book_moves_1);
        assert_eq!(result_2, book_moves_2);
        assert_eq!(result_3, book_moves_3);
    }

    #[test]
    fn it_has_no_changes_after_creation() {
        let book = BookDouble::new();
        let cache = crate::opening_book::cache::Cache::new(book);
        assert_eq!(cache.has_changed(), false);
    }

    #[test]
    fn it_has_changes_after_new_request() {
        let mut book = BookDouble::new();
        let (fen, _) = book.configure(
            Fen::starting_board(),
            vec![BookMove {
                uci: "e2e4".to_owned(),
                frequency: 0.5,
            }],
        );
        let mut cache = crate::opening_book::cache::Cache::new(book);

        // The cache should store the result of this call
        let _ = cache.moves(&fen);

        // Storing the result earlier counts as a change
        assert_eq!(cache.has_changed(), true);
    }

    #[test]
    fn it_has_no_changes_after_saving() {
        let mut data = Vec::new();
        let mut book = BookDouble::new();
        let (fen, _) = book.configure(
            Fen::starting_board(),
            vec![BookMove {
                uci: "e2e4".to_owned(),
                frequency: 0.5,
            }],
        );
        let mut cache = crate::opening_book::cache::Cache::new(book);

        // The cache should store the result of this call
        let _ = cache.moves(&fen);
        let _ = cache.save(&mut data);

        // Saving the cache should reset the change indicator
        assert_eq!(cache.has_changed(), false);
    }

    #[test]
    fn it_restores_itself_by_loading_its_own_save_data() {
        let mut data = Vec::new();
        let mut book_1 = BookDouble::new();
        let mut book_2 = BookDouble::new();
        let (fen_1, book_moves_1) = book_1.configure(
            Fen::starting_board(),
            vec![BookMove {
                uci: "e2e4".to_owned(),
                frequency: 0.5,
            }],
        );
        let (fen_2, book_moves_2) = book_1.configure(
            Fen::new("a b c d e f"),
            vec![BookMove {
                uci: "d2d4".to_owned(),
                frequency: 0.3,
            }],
        );
        let (fen_3, book_moves_3) = book_2.configure(
            Fen::new("a b c D e f"),
            vec![BookMove {
                uci: "c2c4".to_owned(),
                frequency: 0.1,
            }],
        );
        let mut cache = crate::opening_book::cache::Cache::new(book_1);

        // Make some requests to induce caching
        let _ = cache.moves(&fen_1);
        let _ = cache.moves(&fen_2);

        // Save cache and restore a new instance from the saved data
        let _ = cache.save(&mut data);
        let mut cache = crate::opening_book::cache::Cache::new(book_2);
        let _ = cache.load(data.as_slice());

        // Make both new requests and ones that should be cached
        let result_2 = cache.moves(&fen_2);
        let result_3 = cache.moves(&fen_3);
        let result_1 = cache.moves(&fen_1);

        assert_eq!(result_1, book_moves_1);
        assert_eq!(result_2, book_moves_2);
        assert_eq!(result_3, book_moves_3);
    }

    #[test]
    fn it_has_no_changes_after_loading_if_it_didnt_have_changes_before() {
        let mut data = Vec::new();
        let mut book_1 = BookDouble::new();
        let book_2 = BookDouble::new();
        let (fen_1, _) = book_1.configure(
            Fen::starting_board(),
            vec![BookMove {
                uci: "e2e4".to_owned(),
                frequency: 0.5,
            }],
        );
        let mut cache = crate::opening_book::cache::Cache::new(book_1);

        // Make some requests to induce caching
        let _ = cache.moves(&fen_1);

        // Save cache and restore a new instance from the saved data
        let _ = cache.save(&mut data);
        let mut cache = crate::opening_book::cache::Cache::new(book_2);
        let _ = cache.load(data.as_slice());

        assert_eq!(cache.has_changed(), false);
    }

    #[test]
    fn it_has_no_changes_after_loading_if_it_had_changes_before() {
        let mut data = Vec::new();
        let mut book = BookDouble::new();
        let (fen_1, _) = book.configure(
            Fen::starting_board(),
            vec![BookMove {
                uci: "e2e4".to_owned(),
                frequency: 0.5,
            }],
        );
        let (fen_2, _) = book.configure(
            Fen::new("a b c d e f"),
            vec![BookMove {
                uci: "d2d4".to_owned(),
                frequency: 0.3,
            }],
        );
        let mut cache = crate::opening_book::cache::Cache::new(book);

        // Make some requests to induce caching
        let _ = cache.moves(&fen_1);
        // Save cache
        let _ = cache.save(&mut data);
        // Induce new change
        let _ = cache.moves(&fen_2);

        // Load a cache while it has changes
        let _ = cache.load(data.as_slice());

        // Changes should be reset
        assert_eq!(cache.has_changed(), false);
    }
}
