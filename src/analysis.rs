extern crate clap;
extern crate lazy_static;
extern crate serde_json;
extern crate thiserror;
extern crate tokio;

use crate::db;
use std::collections::HashMap;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
  #[error("piece not found on encoded square: {0}")]
  PieceNotFound(char),
  #[error("en passant capture not found on encoded square: {0}")]
  EnPassantPieceNotFound(char),
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Color {
  White,
  Black,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct Square(char);

impl std::fmt::Display for Square {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self.0 {
      'a' => "a1",
      'b' => "b1",
      'c' => "c1",
      'd' => "d1",
      'e' => "e1",
      'f' => "f1",
      'g' => "g1",
      'h' => "h1",
      'i' => "a2",
      'j' => "b2",
      'k' => "c2",
      'l' => "d2",
      'm' => "e2",
      'n' => "f2",
      'o' => "g2",
      'p' => "h2",
      'q' => "a3",
      'r' => "b3",
      's' => "c3",
      't' => "d3",
      'u' => "e3",
      'v' => "f3",
      'w' => "g3",
      'x' => "h3",
      'y' => "a4",
      'z' => "b4",
      'A' => "c4",
      'B' => "d4",
      'C' => "e4",
      'D' => "f4",
      'E' => "g4",
      'F' => "h4",
      'G' => "a5",
      'H' => "b5",
      'I' => "c5",
      'J' => "d5",
      'K' => "e5",
      'L' => "f5",
      'M' => "g5",
      'N' => "h5",
      'O' => "a6",
      'P' => "b6",
      'Q' => "c6",
      'R' => "d6",
      'S' => "e6",
      'T' => "f6",
      'U' => "g6",
      'V' => "h6",
      'W' => "a7",
      'X' => "b7",
      'Y' => "c7",
      'Z' => "d7",
      '0' => "e7",
      '1' => "f7",
      '2' => "g7",
      '3' => "h7",
      '4' => "a8",
      '5' => "b8",
      '6' => "c8",
      '7' => "d8",
      '8' => "e8",
      '9' => "f8",
      '!' => "g8",
      '?' => "h8",
      c => unreachable!(format!("unrecognized board char: {}", c)),
    };
    Ok(write!(f, "{}", s)?)
  }
}

impl From<char> for Square {
  fn from(c: char) -> Self {
    Square(c)
  }
}

impl std::fmt::Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Ok(write!(
      f,
      "{}",
      match self {
        Color::White => "white",
        Color::Black => "black",
      }
    )?)
  }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Piece {
  piece_type: &'static str,
  color: Color,
  value: i32,
}

impl Piece {
  fn with_value(mut self, value: i32) -> Piece {
    self.value = value;
    self
  }
}

impl std::fmt::Display for Piece {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if f.alternate() {
      write!(f, "{} ", self.color)?;
    }
    Ok(write!(f, "{}", self.piece_type)?)
  }
}

pub struct Board {
  piece_map: HashMap<char, Piece>,
  last_move: Option<(&'static str, char, char)>,
  move_num: i32,
}

impl Board {
  pub fn starting() -> Board {
    Board {
      piece_map: maplit::hashmap! {
          'a' => Piece { piece_type: "rook a", color: Color::White, value: 5},
          'b' => Piece { piece_type: "knight b", color: Color::White, value: 3},
          'c' => Piece { piece_type: "bishop c", color: Color::White, value: 3},
          'd' => Piece { piece_type: "queen d", color: Color::White, value: 9},
          'e' => Piece { piece_type: "king e", color: Color::White, value: 0},
          'f' => Piece { piece_type: "bishop f", color: Color::White, value: 3},
          'g' => Piece { piece_type: "knight g", color: Color::White, value: 3},
          'h' => Piece { piece_type: "rook h", color: Color::White, value: 5},
          'i' => Piece { piece_type: "pawn a", color: Color::White, value: 1},
          'j' => Piece { piece_type: "pawn b", color: Color::White, value: 1},
          'k' => Piece { piece_type: "pawn c", color: Color::White, value: 1},
          'l' => Piece { piece_type: "pawn d", color: Color::White, value: 1},
          'm' => Piece { piece_type: "pawn e", color: Color::White, value: 1},
          'n' => Piece { piece_type: "pawn f", color: Color::White, value: 1},
          'o' => Piece { piece_type: "pawn g", color: Color::White, value: 1},
          'p' => Piece { piece_type: "pawn h", color: Color::White, value: 1},

          '4' => Piece { piece_type: "rook a", color: Color::Black, value: 5},
          '5' => Piece { piece_type: "knight b", color: Color::Black, value: 3},
          '6' => Piece { piece_type: "bishop c", color: Color::Black, value: 3},
          '7' => Piece { piece_type: "queen d", color: Color::Black, value: 9},
          '8' => Piece { piece_type: "king e", color: Color::Black, value: 0},
          '9' => Piece { piece_type: "bishop f", color: Color::Black, value: 3},
          '!' => Piece { piece_type: "knight g", color: Color::Black, value: 3},
          '?' => Piece { piece_type: "rook h", color: Color::Black, value: 5},
          'W' => Piece { piece_type: "pawn a", color: Color::Black, value: 1},
          'X' => Piece { piece_type: "pawn b", color: Color::Black, value: 1},
          'Y' => Piece { piece_type: "pawn c", color: Color::Black, value: 1},
          'Z' => Piece { piece_type: "pawn d", color: Color::Black, value: 1},
          '0' => Piece { piece_type: "pawn e", color: Color::Black, value: 1},
          '1' => Piece { piece_type: "pawn f", color: Color::Black, value: 1},
          '2' => Piece { piece_type: "pawn g", color: Color::Black, value: 1},
          '3' => Piece { piece_type: "pawn h", color: Color::Black, value: 1},
      },
      last_move: None,
      move_num: 0,
    }
  }

  pub fn make_move(
    &mut self,
    game_id: &str,
    start: char,
    end: char,
  ) -> Result<db::Move, Error> {
    let move_num = self.move_num;
    self.move_num += 1;
    // Fetch the last move. On all exits to this function, set the last move as
    // this move for the next iteration. We need the last move to detect en
    // passant situations.
    let last_move = std::mem::take(&mut self.last_move);

    // Get the piece at the start location
    let (_loc, moved_piece) =
      self.piece_map.remove_entry(&start).ok_or(Error::PieceNotFound(start))?;

    // Promotion is handled here by looking at the end move; if it is a
    // promotion move, it will have a special character that doesn't correspond
    // to any square on the board. From this char, we can deduce:
    // * the promotion square, which is calculated based on the promotion
    //   direction and the starting square
    // * the piece type that this piece is promoted to
    // We use the promotion piece type to set the value of the piece, without
    // changing the definition of the piece itself. So Pawn on Rank A will
    // always be Pawn on Rank A (so we have continuity in our points tracking),
    // but if it gets promoted to a queen on the board then its capture will be
    // worth 9.
    if let Some(&next_square_lookup) = PROMOTION_DIR.get(&end) {
      let promotion_value = PROMOTION_VALUE.get(&end).unwrap_or_else(|| {
        unreachable!(
          "encoded promotion '{}' has no corresponding piece type",
          end
        )
      });
      let end = next_square_lookup.get(&start).unwrap_or_else(|| {
        unreachable!(
          "piece on encoded square {} has no promotion square",
          start
        )
      });
      let captured_piece = self.piece_map.get(&end).cloned();
      let score = captured_piece
        .as_ref()
        .map(|captured_piece| captured_piece.value)
        .unwrap_or(0);
      self
        .piece_map
        .insert(*end, moved_piece.clone().with_value(*promotion_value));
      self.last_move = Some((moved_piece.piece_type, start, *end));

      return Ok(db::Move {
        game_id: String::from(game_id),
        move_num,
        color: moved_piece.color.to_string(),
        moved_piece: moved_piece.to_string(),
        starting_location: Square::from(start).to_string(),
        ending_location: Square::from(*end).to_string(),
        captured_piece: captured_piece
          .map(|p| p.to_string())
          .unwrap_or_else(|| "".into()),
        capture_score: score,
      });
    }

    // Handle regular (non-promotion) moves
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
        .get(&(start, end))
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
        // Adjust the score for this move's piece based on last move
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
      (Piece { piece_type: "king e", .. }, 'e', 'g') => {
        self.piece_map.insert(end, moved_piece.clone());
        let (_, rook) =
          self.piece_map.remove_entry(&'h').ok_or(Error::PieceNotFound('h'))?;
        self.piece_map.insert('f', rook);
      }
      // White queenside castle
      (Piece { piece_type: "king e", .. }, 'e', 'c') => {
        self.piece_map.insert(end, moved_piece.clone());
        let (_, rook) =
          self.piece_map.remove_entry(&'a').ok_or(Error::PieceNotFound('a'))?;
        self.piece_map.insert('d', rook);
      }
      // Black kingside castle
      (Piece { piece_type: "king e", .. }, '8', '!') => {
        self.piece_map.insert(end, moved_piece.clone());
        let (_, rook) =
          self.piece_map.remove_entry(&'?').ok_or(Error::PieceNotFound('?'))?;
        self.piece_map.insert('9', rook);
      }
      // Black queenside castle
      (Piece { piece_type: "king e", .. }, '8', '6') => {
        self.piece_map.insert(end, moved_piece.clone());
        let (_, rook) =
          self.piece_map.remove_entry(&'4').ok_or(Error::PieceNotFound('4'))?;
        self.piece_map.insert('7', rook);
      }
      // Normal move
      _ => {
        self.piece_map.insert(end, moved_piece.clone());
      }
    }
    self.last_move = Some((moved_piece.piece_type, start, end));
    // Return the starting piece along with its score
    Ok(db::Move {
      game_id: String::from(game_id),
      move_num,
      color: moved_piece.color.to_string(),
      moved_piece: moved_piece.to_string(),
      starting_location: Square::from(start).to_string(),
      ending_location: Square::from(end).to_string(),
      captured_piece: captured_piece
        .map(|p| p.to_string())
        .unwrap_or_else(|| "".into()),
      capture_score: score,
    })
  }
}

lazy_static! {
  static ref PROMOTE_LEFT: HashMap<char, char> = maplit::hashmap! {
    'j' => 'a',
    'k' => 'b',
    'l' => 'c',
    'm' => 'd',
    'n' => 'e',
    'o' => 'f',
    'p' => 'g',
    'X' => '4',
    'Y' => '5',
    'Z' => '6',
    '0' => '7',
    '1' => '8',
    '2' => '9',
    '3' => '!',
  };
  static ref PROMOTE_STRAIGHT: HashMap<char, char> = maplit::hashmap! {
    'i' => 'a',
    'j' => 'b',
    'k' => 'c',
    'l' => 'd',
    'm' => 'e',
    'n' => 'f',
    'o' => 'g',
    'p' => 'h',
    'W' => '4',
    'X' => '5',
    'Y' => '6',
    'Z' => '7',
    '0' => '8',
    '1' => '9',
    '2' => '!',
    '3' => '?',
  };
  static ref PROMOTE_RIGHT: HashMap<char, char> = maplit::hashmap! {
    'i' => 'b',
    'j' => 'c',
    'k' => 'd',
    'l' => 'e',
    'm' => 'f',
    'n' => 'g',
    'o' => 'h',
    'W' => '5',
    'X' => '6',
    'Y' => '7',
    'Z' => '8',
    '0' => '9',
    '1' => '!',
    '2' => '?',
  };
  static ref PROMOTION_DIR: HashMap<char, &'static HashMap<char, char>> = maplit::hashmap! {
    '~' => &*PROMOTE_STRAIGHT,
    '^' => &*PROMOTE_STRAIGHT,
    '_' => &*PROMOTE_STRAIGHT,
    '#' => &*PROMOTE_STRAIGHT,
    '(' => &*PROMOTE_LEFT,
    '{' => &*PROMOTE_LEFT,
    '[' => &*PROMOTE_LEFT,
    '@' => &*PROMOTE_LEFT,
    '}' => &*PROMOTE_RIGHT,
    ')' => &*PROMOTE_RIGHT,
    ']' => &*PROMOTE_RIGHT,
    '$' => &*PROMOTE_RIGHT,
  };
  static ref PROMOTION_VALUE: HashMap<char, i32> = maplit::hashmap! {
    '~' => 9,
    '^' => 3,
    '_' => 5,
    '#' => 3,
    '(' => 3,
    '{' => 9,
    '[' => 5,
    '@' => 3,
    '}' => 9,
    ')' => 3,
    ']' => 5,
    '$' => 3,
  };
  static ref EN_PASSANT_MOVES: HashMap<(char, char), (&'static str, char, char)> = maplit::hashmap! {
    ('y', 'r') => ("pawn b", 'j', 'z'),
    ('z', 's') => ("pawn c", 'k', 'A'),
    ('A', 't') => ("pawn d", 'l', 'B'),
    ('B', 'u') => ("pawn e", 'm', 'C'),
    ('C', 'v') => ("pawn f", 'n', 'D'),
    ('D', 'w') => ("pawn g", 'o', 'E'),
    ('E', 'x') => ("pawn h", 'p', 'F'),

    ('z', 'q') => ("pawn a", 'i', 'y'),
    ('A', 'r') => ("pawn b", 'j', 'z'),
    ('B', 's') => ("pawn c", 'k', 'A'),
    ('C', 't') => ("pawn d", 'l', 'B'),
    ('D', 'u') => ("pawn e", 'm', 'C'),
    ('E', 'x') => ("pawn f", 'n', 'D'),
    ('F', 'w') => ("pawn g", 'o', 'E'),

    ('H', 'O') => ("pawn a", 'W', 'G'),
    ('I', 'P') => ("pawn b", 'X', 'H'),
    ('J', 'Q') => ("pawn c", 'Y', 'I'),
    ('K', 'R') => ("pawn d", 'Z', 'J'),
    ('L', 'S') => ("pawn e", '0', 'K'),
    ('M', 'T') => ("pawn f", '1', 'L'),
    ('N', 'U') => ("pawn g", '2', 'M'),

    ('G', 'P') => ("pawn b", 'X', 'H'),
    ('H', 'Q') => ("pawn c", 'Y', 'I'),
    ('I', 'R') => ("pawn d", 'Z', 'J'),
    ('J', 'S') => ("pawn e", '0', 'K'),
    ('K', 'T') => ("pawn f", '1', 'L'),
    ('L', 'U') => ("pawn g", '2', 'M'),
    ('M', 'V') => ("pawn h", '3', 'N'),
  };
}
