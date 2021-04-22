use std::collections::HashMap;

use thiserror::Error as ThisError;

use crate::db;

#[derive(ThisError, Debug)]
pub enum Error {
  #[error("piece not found on encoded square: {0}")]
  PieceNotFound(Square),
  #[error("en passant capture not found on encoded square: {0}")]
  EnPassantPieceNotFound(Square),
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Color {
  White,
  Black,
}

impl std::fmt::Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Color::White => "white",
        Color::Black => "black",
      }
    )
  }
}

//#[derive(Debug, Hash, Eq, PartialEq, Clone)]
//struct Square(char);
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Square {
  A1,
  B1,
  C1,
  D1,
  E1,
  F1,
  G1,
  H1,
  A2,
  B2,
  C2,
  D2,
  E2,
  F2,
  G2,
  H2,
  A3,
  B3,
  C3,
  D3,
  E3,
  F3,
  G3,
  H3,
  A4,
  B4,
  C4,
  D4,
  E4,
  F4,
  G4,
  H4,
  A5,
  B5,
  C5,
  D5,
  E5,
  F5,
  G5,
  H5,
  A6,
  B6,
  C6,
  D6,
  E6,
  F6,
  G6,
  H6,
  A7,
  B7,
  C7,
  D7,
  E7,
  F7,
  G7,
  H7,
  A8,
  B8,
  C8,
  D8,
  E8,
  F8,
  G8,
  H8,
}

impl std::fmt::Display for Square {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      Square::A1 => "a1",
      Square::B1 => "b1",
      Square::C1 => "c1",
      Square::D1 => "d1",
      Square::E1 => "e1",
      Square::F1 => "f1",
      Square::G1 => "g1",
      Square::H1 => "h1",
      Square::A2 => "a2",
      Square::B2 => "b2",
      Square::C2 => "c2",
      Square::D2 => "d2",
      Square::E2 => "e2",
      Square::F2 => "f2",
      Square::G2 => "g2",
      Square::H2 => "h2",
      Square::A3 => "a3",
      Square::B3 => "b3",
      Square::C3 => "c3",
      Square::D3 => "d3",
      Square::E3 => "e3",
      Square::F3 => "f3",
      Square::G3 => "g3",
      Square::H3 => "h3",
      Square::A4 => "a4",
      Square::B4 => "b4",
      Square::C4 => "c4",
      Square::D4 => "d4",
      Square::E4 => "e4",
      Square::F4 => "f4",
      Square::G4 => "g4",
      Square::H4 => "h4",
      Square::A5 => "a5",
      Square::B5 => "b5",
      Square::C5 => "c5",
      Square::D5 => "d5",
      Square::E5 => "e5",
      Square::F5 => "f5",
      Square::G5 => "g5",
      Square::H5 => "h5",
      Square::A6 => "a6",
      Square::B6 => "b6",
      Square::C6 => "c6",
      Square::D6 => "d6",
      Square::E6 => "e6",
      Square::F6 => "f6",
      Square::G6 => "g6",
      Square::H6 => "h6",
      Square::A7 => "a7",
      Square::B7 => "b7",
      Square::C7 => "c7",
      Square::D7 => "d7",
      Square::E7 => "e7",
      Square::F7 => "f7",
      Square::G7 => "g7",
      Square::H7 => "h7",
      Square::A8 => "a8",
      Square::B8 => "b8",
      Square::C8 => "c8",
      Square::D8 => "d8",
      Square::E8 => "e8",
      Square::F8 => "f8",
      Square::G8 => "g8",
      Square::H8 => "h8",
    };
    write!(f, "{}", s)
  }
}

impl From<char> for &Square {
  fn from(c: char) -> Self {
    match c {
      'a' => &Square::A1,
      'b' => &Square::B1,
      'c' => &Square::C1,
      'd' => &Square::D1,
      'e' => &Square::E1,
      'f' => &Square::F1,
      'g' => &Square::G1,
      'h' => &Square::H1,
      'i' => &Square::A2,
      'j' => &Square::B2,
      'k' => &Square::C2,
      'l' => &Square::D2,
      'm' => &Square::E2,
      'n' => &Square::F2,
      'o' => &Square::G2,
      'p' => &Square::H2,
      'q' => &Square::A3,
      'r' => &Square::B3,
      's' => &Square::C3,
      't' => &Square::D3,
      'u' => &Square::E3,
      'v' => &Square::F3,
      'w' => &Square::G3,
      'x' => &Square::H3,
      'y' => &Square::A4,
      'z' => &Square::B4,
      'A' => &Square::C4,
      'B' => &Square::D4,
      'C' => &Square::E4,
      'D' => &Square::F4,
      'E' => &Square::G4,
      'F' => &Square::H4,
      'G' => &Square::A5,
      'H' => &Square::B5,
      'I' => &Square::C5,
      'J' => &Square::D5,
      'K' => &Square::E5,
      'L' => &Square::F5,
      'M' => &Square::G5,
      'N' => &Square::H5,
      'O' => &Square::A6,
      'P' => &Square::B6,
      'Q' => &Square::C6,
      'R' => &Square::D6,
      'S' => &Square::E6,
      'T' => &Square::F6,
      'U' => &Square::G6,
      'V' => &Square::H6,
      'W' => &Square::A7,
      'X' => &Square::B7,
      'Y' => &Square::C7,
      'Z' => &Square::D7,
      '0' => &Square::E7,
      '1' => &Square::F7,
      '2' => &Square::G7,
      '3' => &Square::H7,
      '4' => &Square::A8,
      '5' => &Square::B8,
      '6' => &Square::C8,
      '7' => &Square::D8,
      '8' => &Square::E8,
      '9' => &Square::F8,
      '!' => &Square::G8,
      '?' => &Square::H8,
      c => unreachable!(format!("unrecognized board char: {}", c)),
    }
  }
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct Piece {
  piece_type: &'static str,
  color: Color,
  value: i32,
}

impl Piece {
  fn with_value(mut self, value: Option<i32>) -> Piece {
    self.value = match value {
      None => self.value,
      Some(v) => v,
    };
    self
  }
}

impl std::fmt::Display for Piece {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if f.alternate() {
      write!(f, "{} ", self.color)?;
    }
    write!(f, "{}", self.piece_type)
  }
}

pub struct Board {
  piece_map: HashMap<Square, Piece>,
  last_move: Option<(&'static str, Square, Square)>,
  move_num: i32,
}

impl Board {
  pub fn starting() -> Board {
    Board {
      piece_map: maplit::hashmap! {
          Square::A1 => Piece { piece_type: "rook a", color: Color::White, value: 5},
          Square::B1 => Piece { piece_type: "knight b", color: Color::White, value: 3},
          Square::C1 => Piece { piece_type: "bishop c", color: Color::White, value: 3},
          Square::D1 => Piece { piece_type: "queen d", color: Color::White, value: 9},
          Square::E1 => Piece { piece_type: "king e", color: Color::White, value: 0},
          Square::F1 => Piece { piece_type: "bishop f", color: Color::White, value: 3},
          Square::G1 => Piece { piece_type: "knight g", color: Color::White, value: 3},
          Square::H1 => Piece { piece_type: "rook h", color: Color::White, value: 5},
          Square::A2 => Piece { piece_type: "pawn a", color: Color::White, value: 1},
          Square::B2 => Piece { piece_type: "pawn b", color: Color::White, value: 1},
          Square::C2 => Piece { piece_type: "pawn c", color: Color::White, value: 1},
          Square::D2 => Piece { piece_type: "pawn d", color: Color::White, value: 1},
          Square::E2 => Piece { piece_type: "pawn e", color: Color::White, value: 1},
          Square::F2 => Piece { piece_type: "pawn f", color: Color::White, value: 1},
          Square::G2 => Piece { piece_type: "pawn g", color: Color::White, value: 1},
          Square::H2 => Piece { piece_type: "pawn h", color: Color::White, value: 1},

          Square::A8 => Piece { piece_type: "rook a", color: Color::Black, value: 5},
          Square::B8 => Piece { piece_type: "knight b", color: Color::Black, value: 3},
          Square::C8 => Piece { piece_type: "bishop c", color: Color::Black, value: 3},
          Square::D8 => Piece { piece_type: "queen d", color: Color::Black, value: 9},
          Square::E8 => Piece { piece_type: "king e", color: Color::Black, value: 0},
          Square::F8 => Piece { piece_type: "bishop f", color: Color::Black, value: 3},
          Square::G8 => Piece { piece_type: "knight g", color: Color::Black, value: 3},
          Square::H8 => Piece { piece_type: "rook h", color: Color::Black, value: 5},
          Square::A7 => Piece { piece_type: "pawn a", color: Color::Black, value: 1},
          Square::B7 => Piece { piece_type: "pawn b", color: Color::Black, value: 1},
          Square::C7 => Piece { piece_type: "pawn c", color: Color::Black, value: 1},
          Square::D7 => Piece { piece_type: "pawn d", color: Color::Black, value: 1},
          Square::E7 => Piece { piece_type: "pawn e", color: Color::Black, value: 1},
          Square::F7 => Piece { piece_type: "pawn f", color: Color::Black, value: 1},
          Square::G7 => Piece { piece_type: "pawn g", color: Color::Black, value: 1},
          Square::H7 => Piece { piece_type: "pawn h", color: Color::Black, value: 1},
      },
      last_move: None,
      move_num: 0,
    }
  }

  pub fn make_move(
    &mut self,
    start: &Square,
    end: &Square,
    promotion_value: Option<i32>,
  ) -> Result<db::Move, Error> {
    let move_num = self.move_num;
    self.move_num += 1;
    // Fetch the last move. On all exits to this function, set the last move as
    // this move for the next iteration. We need the last move to detect en
    // passant situations.
    let last_move = std::mem::take(&mut self.last_move);

    // Get the piece at the start location
    let (_loc, moved_piece) = self
      .piece_map
      .remove_entry(start)
      .ok_or_else(|| Error::PieceNotFound(start.clone()))?;

    // Handle regular moves
    // Get the piece at the end location; if there is one, the score is the
    // value of the piece.
    // If this is an en-passant capture, this calculation will be zero but will
    // be updated below.
    let mut captured_piece = self.piece_map.get(&end).cloned();

    // Handle en passant
    // If the last move was a pawn double-move and this move is a pawn capture
    // corresponding to that particular pawn double-move, then this was an en
    // passant capture. The piece moved last move is removed from the board, and
    // the piece moved this turn scores points.
    if let Some(last_move) = last_move {
      if EN_PASSANT_MOVES
        .get(&(start.clone(), end.clone()))
        .map(|f| f.0 == last_move.0 && f.1 == last_move.1 && f.2 == last_move.2)
        == Some(true)
      {
        println!("EN PASSANT");
        // Remove the piece on the last move's end square
        captured_piece = Some(
          self
            .piece_map
            .remove(&last_move.2)
            .ok_or(Error::EnPassantPieceNotFound(last_move.2))?,
        );
      }
    }
    let score = captured_piece.as_ref().map(|p| p.value).unwrap_or(0);

    // Castling is handled here by seeing if we see the king jump 2 squares
    // in one of the possible castling scenarios. If this happens, we need
    // to be sure to update the rook as well, as its movement is implied;
    // failure to do so means that the rook won't be found at its expected
    // square when its moved later.
    match (moved_piece.clone(), start, end) {
      // White kingside castle
      (Piece { piece_type: "king e", .. }, Square::E1, Square::G1) => {
        self.piece_map.insert(end.clone(), moved_piece.clone());
        let (_, rook) = self
          .piece_map
          .remove_entry(&Square::H1)
          .ok_or(Error::PieceNotFound(Square::H1))?;
        self.piece_map.insert(Square::F1, rook);
      }
      // White queenside castle
      (Piece { piece_type: "king e", .. }, Square::E1, Square::C1) => {
        self.piece_map.insert(end.clone(), moved_piece.clone());
        let (_, rook) = self
          .piece_map
          .remove_entry(&Square::A1)
          .ok_or(Error::PieceNotFound(Square::A1))?;
        self.piece_map.insert(Square::D1, rook);
      }
      // Black kingside castle
      (Piece { piece_type: "king e", .. }, Square::E8, Square::G8) => {
        self.piece_map.insert(end.clone(), moved_piece.clone());
        let (_, rook) = self
          .piece_map
          .remove_entry(&Square::H8)
          .ok_or(Error::PieceNotFound(Square::H8))?;
        self.piece_map.insert(Square::F8, rook);
      }
      // Black queenside castle
      (Piece { piece_type: "king e", .. }, Square::E8, Square::C8) => {
        self.piece_map.insert(end.clone(), moved_piece.clone());
        let (_, rook) = self
          .piece_map
          .remove_entry(&Square::A8)
          .ok_or(Error::PieceNotFound(Square::A8))?;
        self.piece_map.insert(Square::D8, rook);
      }
      // Normal move
      // We use the promotion piece type to set the value of the piece, without
      // changing the definition of the piece itself. So Pawn on Rank A will
      // always be Pawn on Rank A (so we have continuity in our points tracking),
      // but if it gets promoted to a queen on the board then its capture will be
      // worth 9.
      _ => {
        self
          .piece_map
          .insert(end.clone(), moved_piece.clone().with_value(promotion_value));
      }
    }
    self.last_move = Some((moved_piece.piece_type, start.clone(), end.clone()));
    // Return the starting piece along with its score
    Ok(db::Move {
      move_num,
      color: moved_piece.color.to_string(),
      moved_piece: moved_piece.to_string(),
      starting_location: start.to_string(),
      ending_location: end.to_string(),
      captured_piece: captured_piece
        .map(|p| p.to_string())
        .unwrap_or_else(|| "".into()),
      capture_score: score,
    })
  }
}

lazy_static! {
  static ref EN_PASSANT_MOVES: HashMap<(Square, Square), (&'static str, Square, Square)> = maplit::hashmap! {
    (Square::A4, Square::B3) => ("pawn b", Square::B2, Square::B4),
    (Square::B4, Square::C3) => ("pawn c", Square::C2, Square::C4),
    (Square::C4, Square::D3) => ("pawn d", Square::D2, Square::D4),
    (Square::D4, Square::E3) => ("pawn e", Square::E2, Square::E4),
    (Square::E4, Square::F3) => ("pawn f", Square::F2, Square::F4),
    (Square::F4, Square::G3) => ("pawn g", Square::G2, Square::G4),
    (Square::G4, Square::H3) => ("pawn h", Square::H2, Square::H4),

    (Square::B4, Square::A3) => ("pawn a", Square::A2, Square::A4),
    (Square::C4, Square::B3) => ("pawn b", Square::B2, Square::B4),
    (Square::D4, Square::C3) => ("pawn c", Square::C2, Square::C4),
    (Square::E4, Square::D3) => ("pawn d", Square::D2, Square::D4),
    (Square::F4, Square::E3) => ("pawn e", Square::E2, Square::E4),
    (Square::G4, Square::F3) => ("pawn f", Square::F2, Square::F4),
    (Square::H4, Square::G3) => ("pawn g", Square::G2, Square::G4),

    (Square::B5, Square::A6) => ("pawn a", Square::A7, Square::A5),
    (Square::C5, Square::B6) => ("pawn b", Square::B7, Square::B5),
    (Square::D5, Square::C6) => ("pawn c", Square::C7, Square::C5),
    (Square::E5, Square::D6) => ("pawn d", Square::D7, Square::D5),
    (Square::F5, Square::E6) => ("pawn e", Square::E7, Square::E5),
    (Square::G5, Square::F6) => ("pawn f", Square::F7, Square::F5),
    (Square::H5, Square::G6) => ("pawn g", Square::G7, Square::G5),

    (Square::A5, Square::B6) => ("pawn b", Square::B7, Square::B5),
    (Square::B5, Square::C6) => ("pawn c", Square::C7, Square::C5),
    (Square::C5, Square::D6) => ("pawn d", Square::D7, Square::D5),
    (Square::D5, Square::E6) => ("pawn e", Square::E7, Square::E5),
    (Square::E5, Square::F6) => ("pawn f", Square::F7, Square::F5),
    (Square::F5, Square::G6) => ("pawn g", Square::G7, Square::G5),
    (Square::G5, Square::H6) => ("pawn h", Square::H7, Square::H5),
  };
}
