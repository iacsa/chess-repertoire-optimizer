use chess_pgn_parser as cpg;
use chess_pgn_parser::{Move, Square};
use pleco::core::piece_move::BitMove;
use pleco::core::{sq::SQ, File, Piece, Rank};
use pleco::Board;

pub fn move_matches_bitmove(mv: &Move, bmv: BitMove, board: &Board) -> bool {
    match mv {
        Move::CastleKingside => bmv.is_king_castle(),
        Move::CastleQueenside => bmv.is_queen_castle(),
        Move::BasicMove {
            piece,
            to,
            from,
            is_capture,
            promoted_to,
        } => {
            *is_capture == bmv.is_capture()
                && equal_file(to, bmv.get_dest())
                && equal_rank(to, bmv.get_dest())
                && equal_piece(*piece, board.piece_at_sq(bmv.get_src()))
                && equal_file(from, bmv.get_src())
                && equal_rank(from, bmv.get_src())
                && equal_promotion(*promoted_to, bmv)
        }
    }
}

fn equal_file(sq1: &Square, sq2: SQ) -> bool {
    match sq1 {
        Square::A1
        | Square::A2
        | Square::A3
        | Square::A4
        | Square::A5
        | Square::A6
        | Square::A7
        | Square::A8
        | Square::AX => sq2.file() == File::A,
        Square::B1
        | Square::B2
        | Square::B3
        | Square::B4
        | Square::B5
        | Square::B6
        | Square::B7
        | Square::B8
        | Square::BX => sq2.file() == File::B,
        Square::C1
        | Square::C2
        | Square::C3
        | Square::C4
        | Square::C5
        | Square::C6
        | Square::C7
        | Square::C8
        | Square::CX => sq2.file() == File::C,
        Square::D1
        | Square::D2
        | Square::D3
        | Square::D4
        | Square::D5
        | Square::D6
        | Square::D7
        | Square::D8
        | Square::DX => sq2.file() == File::D,
        Square::E1
        | Square::E2
        | Square::E3
        | Square::E4
        | Square::E5
        | Square::E6
        | Square::E7
        | Square::E8
        | Square::EX => sq2.file() == File::E,
        Square::F1
        | Square::F2
        | Square::F3
        | Square::F4
        | Square::F5
        | Square::F6
        | Square::F7
        | Square::F8
        | Square::FX => sq2.file() == File::F,
        Square::G1
        | Square::G2
        | Square::G3
        | Square::G4
        | Square::G5
        | Square::G6
        | Square::G7
        | Square::G8
        | Square::GX => sq2.file() == File::G,
        Square::H1
        | Square::H2
        | Square::H3
        | Square::H4
        | Square::H5
        | Square::H6
        | Square::H7
        | Square::H8
        | Square::HX => sq2.file() == File::H,
        _ => true, // Unknown file
    }
}

fn equal_rank(sq1: &Square, sq2: SQ) -> bool {
    match sq1 {
        Square::A1
        | Square::B1
        | Square::C1
        | Square::D1
        | Square::E1
        | Square::F1
        | Square::G1
        | Square::H1
        | Square::X1 => sq2.rank() == Rank::R1,
        Square::A2
        | Square::B2
        | Square::C2
        | Square::D2
        | Square::E2
        | Square::F2
        | Square::G2
        | Square::H2
        | Square::X2 => sq2.rank() == Rank::R2,
        Square::A3
        | Square::B3
        | Square::C3
        | Square::D3
        | Square::E3
        | Square::F3
        | Square::G3
        | Square::H3
        | Square::X3 => sq2.rank() == Rank::R3,
        Square::A4
        | Square::B4
        | Square::C4
        | Square::D4
        | Square::E4
        | Square::F4
        | Square::G4
        | Square::H4
        | Square::X4 => sq2.rank() == Rank::R4,
        Square::A5
        | Square::B5
        | Square::C5
        | Square::D5
        | Square::E5
        | Square::F5
        | Square::G5
        | Square::H5
        | Square::X5 => sq2.rank() == Rank::R5,
        Square::A6
        | Square::B6
        | Square::C6
        | Square::D6
        | Square::E6
        | Square::F6
        | Square::G6
        | Square::H6
        | Square::X6 => sq2.rank() == Rank::R6,
        Square::A7
        | Square::B7
        | Square::C7
        | Square::D7
        | Square::E7
        | Square::F7
        | Square::G7
        | Square::H7
        | Square::X7 => sq2.rank() == Rank::R7,
        Square::A8
        | Square::B8
        | Square::C8
        | Square::D8
        | Square::E8
        | Square::F8
        | Square::G8
        | Square::H8
        | Square::X8 => sq2.rank() == Rank::R8,
        _ => true, // Unknown rank
    }
}

fn equal_piece(p1: cpg::Piece, p2: Piece) -> bool {
    match p1 {
        cpg::Piece::Bishop => p2 == Piece::WhiteBishop || p2 == Piece::BlackBishop,
        cpg::Piece::Knight => p2 == Piece::WhiteKnight || p2 == Piece::BlackKnight,
        cpg::Piece::Queen => p2 == Piece::WhiteQueen || p2 == Piece::BlackQueen,
        cpg::Piece::Rook => p2 == Piece::WhiteRook || p2 == Piece::BlackRook,
        cpg::Piece::King => p2 == Piece::WhiteKing || p2 == Piece::BlackKing,
        cpg::Piece::Pawn => p2 == Piece::WhitePawn || p2 == Piece::BlackPawn,
    }
}

fn equal_promotion(po: Option<cpg::Piece>, bmv: BitMove) -> bool {
    if let Some(p1) = po {
        if !bmv.is_promo() {
            return false;
        }
        let p2 = bmv.promo_piece();
        match p1 {
            cpg::Piece::Bishop => p2 == pleco::core::PieceType::B,
            cpg::Piece::Knight => p2 == pleco::core::PieceType::N,
            cpg::Piece::Queen => p2 == pleco::core::PieceType::Q,
            cpg::Piece::Rook => p2 == pleco::core::PieceType::R,
            _ => false,
        }
    } else {
        !bmv.is_promo()
    }
}
