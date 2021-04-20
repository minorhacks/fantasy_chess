extern crate clap;
extern crate lazy_static;
extern crate serde_json;
extern crate thiserror;
extern crate tokio;

use crate::api;
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
enum PieceType {
  King,
  Queen,
  Bishop,
  Knight,
  Rook,
  Pawn,
}

impl PieceType {
  fn value(&self) -> i32 {
    match self {
      PieceType::King => 0, // TODO: what should be the value of checkmate?
      PieceType::Queen => 9,
      PieceType::Bishop => 3,
      PieceType::Knight => 3,
      PieceType::Rook => 5,
      PieceType::Pawn => 1,
    }
  }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum File {
  A,
  B,
  C,
  D,
  E,
  F,
  G,
  H,
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
  piece_type: PieceType,
  color: Color,
  file: File,
  promoted_value: Option<i32>,
}

impl Piece {
  fn new(piece_type: PieceType, color: Color, file: File) -> Piece {
    Piece { piece_type, color, file, promoted_value: None }
  }

  fn with_promotion(mut self, piece_type: &PieceType) -> Piece {
    self.promoted_value = Some(piece_type.value());
    self
  }

  fn value(&self) -> i32 {
    // Piece defers to the value of its type, unless it's been promoted, in
    // which case the value is overridden in promoted_value.
    self.promoted_value.unwrap_or_else(|| self.piece_type.value())
  }
}

impl std::fmt::Debug for Piece {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?} {:?}", self.color, self.piece_type)?;
    match self.piece_type {
      PieceType::Bishop
      | PieceType::Knight
      | PieceType::Rook
      | PieceType::Pawn => write!(f, " {:?}", self.file)?,
      _ => (),
    }
    Ok(())
  }
}

impl std::fmt::Display for Piece {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if f.alternate() {
      write!(f, "{:?} ", self.color)?;
    }
    write!(f, "{:?}", self.piece_type)?;
    match self.piece_type {
      PieceType::Bishop
      | PieceType::Knight
      | PieceType::Rook
      | PieceType::Pawn => write!(f, " {:?}", self.file)?,
      _ => (),
    }
    Ok(())
  }
}

pub struct Board {
  piece_map: HashMap<char, Piece>,
  last_move: Option<(PieceType, char, char)>,
  move_num: i32,
}

impl Board {
  pub fn starting() -> Board {
    Board {
      piece_map: maplit::hashmap! {
          'a' => Piece::new(PieceType::Rook, Color::White, File::A),
          'b' => Piece::new(PieceType::Knight, Color::White, File::B),
          'c' => Piece::new(PieceType::Bishop, Color::White, File::C),
          'd' => Piece::new(PieceType::Queen, Color::White, File::D),
          'e' => Piece::new(PieceType::King, Color::White, File::E),
          'f' => Piece::new(PieceType::Bishop, Color::White, File::F),
          'g' => Piece::new(PieceType::Knight, Color::White, File::G),
          'h' => Piece::new(PieceType::Rook, Color::White, File::H),
          'i' => Piece::new(PieceType::Pawn, Color::White, File::A),
          'j' => Piece::new(PieceType::Pawn, Color::White, File::B),
          'k' => Piece::new(PieceType::Pawn, Color::White, File::C),
          'l' => Piece::new(PieceType::Pawn, Color::White, File::D),
          'm' => Piece::new(PieceType::Pawn, Color::White, File::E),
          'n' => Piece::new(PieceType::Pawn, Color::White, File::F),
          'o' => Piece::new(PieceType::Pawn, Color::White, File::G),
          'p' => Piece::new(PieceType::Pawn, Color::White, File::H),

          '4' => Piece::new(PieceType::Rook, Color::Black, File::A),
          '5' => Piece::new(PieceType::Knight, Color::Black, File::B),
          '6' => Piece::new(PieceType::Bishop, Color::Black, File::C),
          '7' => Piece::new(PieceType::Queen, Color::Black, File::D),
          '8' => Piece::new(PieceType::King, Color::Black, File::E),
          '9' => Piece::new(PieceType::Bishop, Color::Black, File::F),
          '!' => Piece::new(PieceType::Knight, Color::Black, File::G),
          '?' => Piece::new(PieceType::Rook, Color::Black, File::H),
          'W' => Piece::new(PieceType::Pawn, Color::Black, File::A),
          'X' => Piece::new(PieceType::Pawn, Color::Black, File::B),
          'Y' => Piece::new(PieceType::Pawn, Color::Black, File::C),
          'Z' => Piece::new(PieceType::Pawn, Color::Black, File::D),
          '0' => Piece::new(PieceType::Pawn, Color::Black, File::E),
          '1' => Piece::new(PieceType::Pawn, Color::Black, File::F),
          '2' => Piece::new(PieceType::Pawn, Color::Black, File::G),
          '3' => Piece::new(PieceType::Pawn, Color::Black, File::H),
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
      let promotion = PROMOTION_TYPE.get(&end).unwrap_or_else(|| {
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
        .map(|captured_piece| captured_piece.value())
        .unwrap_or(0);
      self
        .piece_map
        .insert(*end, moved_piece.clone().with_promotion(promotion));
      self.last_move = Some((moved_piece.piece_type.clone(), start, *end));

      return Ok(db::Move {
        game_id: String::from(game_id),
        move_num: self.move_num,
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
    let score = captured_piece.as_ref().map(|p| p.value()).unwrap_or(0);

    // Castling is handled here by seeing if we see the king jump 2 squares
    // in one of the possible castling scenarios. If this happens, we need
    // to be sure to update the rook as well, as its movement is implied;
    // failure to do so means that the rook won't be found at its expected
    // square when its moved later.
    match (moved_piece.clone(), start, end) {
      // White kingside castle
      (Piece { piece_type: PieceType::King, .. }, 'e', 'g') => {
        self.piece_map.insert(end, moved_piece.clone());
        let (_, rook) =
          self.piece_map.remove_entry(&'h').ok_or(Error::PieceNotFound('h'))?;
        self.piece_map.insert('f', rook);
      }
      // White queenside castle
      (Piece { piece_type: PieceType::King, .. }, 'e', 'c') => {
        self.piece_map.insert(end, moved_piece.clone());
        let (_, rook) =
          self.piece_map.remove_entry(&'a').ok_or(Error::PieceNotFound('a'))?;
        self.piece_map.insert('d', rook);
      }
      // Black kingside castle
      (Piece { piece_type: PieceType::King, .. }, '8', '!') => {
        self.piece_map.insert(end, moved_piece.clone());
        let (_, rook) =
          self.piece_map.remove_entry(&'?').ok_or(Error::PieceNotFound('?'))?;
        self.piece_map.insert('9', rook);
      }
      // Black queenside castle
      (Piece { piece_type: PieceType::King, .. }, '8', '6') => {
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
    self.last_move = Some((moved_piece.piece_type.clone(), start, end));
    // Return the starting piece along with its score
    Ok(db::Move {
      game_id: String::from(game_id),
      move_num: self.move_num,
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

  fn move_and_score(
    &mut self,
    start: char,
    end: char,
  ) -> Result<(Piece, i32), Error> {
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
      let promotion = PROMOTION_TYPE.get(&end).unwrap_or_else(|| {
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
      let score = self
        .piece_map
        .get(&end)
        .map(|captured_piece| captured_piece.value())
        .unwrap_or(0);
      self
        .piece_map
        .insert(*end, moved_piece.clone().with_promotion(promotion));
      self.last_move = Some((moved_piece.piece_type.clone(), start, *end));
      return Ok((moved_piece, score));
    }

    // Handle regular (non-promotion) moves
    // Get the piece at the end location; if there is one, the score is the
    // value of the piece.
    // If this is an en-passant capture, this calculation will be zero but will
    // be updated below.
    let mut score = self
      .piece_map
      .get(&end)
      .map(|captured_piece| captured_piece.value())
      .unwrap_or(0);

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
        let captured = self
          .piece_map
          .remove(&last_move.2)
          .ok_or(Error::EnPassantPieceNotFound(last_move.2))?;
        // Adjust the score for this move's piece based on last move
        score = captured.value();
      }
    }

    // Castling is handled here by seeing if we see the king jump 2 squares
    // in one of the possible castling scenarios. If this happens, we need
    // to be sure to update the rook as well, as its movement is implied;
    // failure to do so means that the rook won't be found at its expected
    // square when its moved later.
    match (moved_piece.clone(), start, end) {
      // White kingside castle
      (Piece { piece_type: PieceType::King, .. }, 'e', 'g') => {
        self.piece_map.insert(end, moved_piece.clone());
        let (_, rook) =
          self.piece_map.remove_entry(&'h').ok_or(Error::PieceNotFound('h'))?;
        self.piece_map.insert('f', rook);
      }
      // White queenside castle
      (Piece { piece_type: PieceType::King, .. }, 'e', 'c') => {
        self.piece_map.insert(end, moved_piece.clone());
        let (_, rook) =
          self.piece_map.remove_entry(&'a').ok_or(Error::PieceNotFound('a'))?;
        self.piece_map.insert('d', rook);
      }
      // Black kingside castle
      (Piece { piece_type: PieceType::King, .. }, '8', '!') => {
        self.piece_map.insert(end, moved_piece.clone());
        let (_, rook) =
          self.piece_map.remove_entry(&'?').ok_or(Error::PieceNotFound('?'))?;
        self.piece_map.insert('9', rook);
      }
      // Black queenside castle
      (Piece { piece_type: PieceType::King, .. }, '8', '6') => {
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
    self.last_move = Some((moved_piece.piece_type.clone(), start, end));
    // Return the starting piece along with its score
    Ok((moved_piece, score))
  }
}

#[derive(Debug)]
pub struct PieceScore {
  scores: HashMap<Piece, i32>,
}

lazy_static! {
  static ref WHITE_PIECES: Vec<Piece> = vec![
    Piece::new(PieceType::King, Color::White, File::E),
    Piece::new(PieceType::Queen, Color::White, File::D),
    Piece::new(PieceType::Rook, Color::White, File::A),
    Piece::new(PieceType::Rook, Color::White, File::H),
    Piece::new(PieceType::Knight, Color::White, File::B),
    Piece::new(PieceType::Knight, Color::White, File::G),
    Piece::new(PieceType::Bishop, Color::White, File::C),
    Piece::new(PieceType::Bishop, Color::White, File::F),
    Piece::new(PieceType::Pawn, Color::White, File::A),
    Piece::new(PieceType::Pawn, Color::White, File::B),
    Piece::new(PieceType::Pawn, Color::White, File::C),
    Piece::new(PieceType::Pawn, Color::White, File::D),
    Piece::new(PieceType::Pawn, Color::White, File::E),
    Piece::new(PieceType::Pawn, Color::White, File::F),
    Piece::new(PieceType::Pawn, Color::White, File::G),
    Piece::new(PieceType::Pawn, Color::White, File::H),
  ];
  static ref BLACK_PIECES: Vec<Piece> = vec![
    Piece::new(PieceType::King, Color::Black, File::E),
    Piece::new(PieceType::Queen, Color::Black, File::D),
    Piece::new(PieceType::Rook, Color::Black, File::A),
    Piece::new(PieceType::Rook, Color::Black, File::H),
    Piece::new(PieceType::Knight, Color::Black, File::B),
    Piece::new(PieceType::Knight, Color::Black, File::G),
    Piece::new(PieceType::Bishop, Color::Black, File::C),
    Piece::new(PieceType::Bishop, Color::Black, File::F),
    Piece::new(PieceType::Pawn, Color::Black, File::A),
    Piece::new(PieceType::Pawn, Color::Black, File::B),
    Piece::new(PieceType::Pawn, Color::Black, File::C),
    Piece::new(PieceType::Pawn, Color::Black, File::D),
    Piece::new(PieceType::Pawn, Color::Black, File::E),
    Piece::new(PieceType::Pawn, Color::Black, File::F),
    Piece::new(PieceType::Pawn, Color::Black, File::G),
    Piece::new(PieceType::Pawn, Color::Black, File::H),
  ];
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
  static ref PROMOTION_TYPE: HashMap<char, PieceType> = maplit::hashmap! {
    '~' => PieceType::Queen,
    '^' => PieceType::Knight,
    '_' => PieceType::Rook,
    '#' => PieceType::Bishop,
    '(' => PieceType::Knight,
    '{' => PieceType::Queen,
    '[' => PieceType::Rook,
    '@' => PieceType::Bishop,
    '}' => PieceType::Queen,
    ')' => PieceType::Knight,
    ']' => PieceType::Rook,
    '$' => PieceType::Bishop,
  };
  static ref EN_PASSANT_MOVES: HashMap<(char, char), (PieceType, char, char)> = maplit::hashmap! {
    ('y', 'r') => (PieceType::Pawn, 'j', 'z'),
    ('z', 's') => (PieceType::Pawn, 'k', 'A'),
    ('A', 't') => (PieceType::Pawn, 'l', 'B'),
    ('B', 'u') => (PieceType::Pawn, 'm', 'C'),
    ('C', 'v') => (PieceType::Pawn, 'n', 'D'),
    ('D', 'w') => (PieceType::Pawn, 'o', 'E'),
    ('E', 'x') => (PieceType::Pawn, 'p', 'F'),

    ('z', 'q') => (PieceType::Pawn, 'i', 'y'),
    ('A', 'r') => (PieceType::Pawn, 'j', 'z'),
    ('B', 's') => (PieceType::Pawn, 'k', 'A'),
    ('C', 't') => (PieceType::Pawn, 'l', 'B'),
    ('D', 'u') => (PieceType::Pawn, 'm', 'C'),
    ('E', 'x') => (PieceType::Pawn, 'n', 'D'),
    ('F', 'w') => (PieceType::Pawn, 'o', 'E'),

    ('H', 'O') => (PieceType::Pawn, 'W', 'G'),
    ('I', 'P') => (PieceType::Pawn, 'X', 'H'),
    ('J', 'Q') => (PieceType::Pawn, 'Y', 'I'),
    ('K', 'R') => (PieceType::Pawn, 'Z', 'J'),
    ('L', 'S') => (PieceType::Pawn, '0', 'K'),
    ('M', 'T') => (PieceType::Pawn, '1', 'L'),
    ('N', 'U') => (PieceType::Pawn, '2', 'M'),

    ('G', 'P') => (PieceType::Pawn, 'X', 'H'),
    ('H', 'Q') => (PieceType::Pawn, 'Y', 'I'),
    ('I', 'R') => (PieceType::Pawn, 'Z', 'J'),
    ('J', 'S') => (PieceType::Pawn, '0', 'K'),
    ('K', 'T') => (PieceType::Pawn, '1', 'L'),
    ('L', 'U') => (PieceType::Pawn, '2', 'M'),
    ('M', 'V') => (PieceType::Pawn, '3', 'N'),
  };
}

impl std::fmt::Display for PieceScore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "White:")?;
    WHITE_PIECES
      .iter()
      .map(|p| self.scores.get_key_value(p).or(Some((p, &0))).unwrap())
      .try_for_each(|(name, score)| writeln!(f, "\t{} - {}", name, score))?;
    writeln!(f)?;
    writeln!(f, "Black:")?;
    BLACK_PIECES
      .iter()
      .map(|p| self.scores.get_key_value(p).or(Some((p, &0))).unwrap())
      .try_for_each(|(name, score)| writeln!(f, "\t{} - {}", name, score))?;
    Ok(())
  }
}

pub fn score_game(game: &api::Game) -> Result<PieceScore, Error> {
  let mut final_score = PieceScore { scores: HashMap::new() };
  let mut board = Board::starting();
  let moves = &game.move_list.clone();
  let mut move_list = moves.chars().fuse();

  while let Some(start) = move_list.next() {
    if let Some(end) = move_list.next() {
      let (piece, score) = board.move_and_score(start, end)?;
      *final_score.scores.entry(piece).or_insert(0) += score;
    } else {
      panic!();
    }
  }
  Ok(final_score)
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::api;

  fn strip_space(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
  }

  #[test]
  fn test_score_game_simple() {
    let game_7545694171 = api::Game {
      move_list: strip_space(
        r#"
mC 0K
bs 1L
lt !T
gv 9I
sy 70
cM 8!
iq ZJ
jz IP
tB 3V
MT 9T
BK TQ
fH QU
eg 6S
ks WG
vB PB
dc 4W
cd W4
dB 0M
ad Mo
"#,
      ),
    };
    let score = score_game(&game_7545694171);
    assert!(score.is_ok());
    let score = score.unwrap();
    assert_eq!(
      Some(&3),
      score.scores.get(&Piece::new(PieceType::Bishop, Color::White, File::C))
    );
    assert_eq!(
      Some(&3),
      score.scores.get(&Piece::new(PieceType::Rook, Color::Black, File::H))
    );
    assert_eq!(
      Some(&1),
      score.scores.get(&Piece::new(PieceType::Pawn, Color::White, File::D))
    );
    assert_eq!(
      Some(&3),
      score.scores.get(&Piece::new(PieceType::Bishop, Color::Black, File::F))
    );
    assert_eq!(
      Some(&3),
      score.scores.get(&Piece::new(PieceType::Queen, Color::White, File::D))
    );
    assert_eq!(
      Some(&1),
      score.scores.get(&Piece::new(PieceType::Queen, Color::Black, File::D))
    );
  }

  #[test]
  fn test_score_game_en_passant_white_promote_queen() {
    let game_9695070671 = api::Game {
      move_list: strip_space(
        r#"
mC 0K
lB 3N
BJ YI
JQ XH
fH WG
gv ZJ
bs 1L
cD 2M
dl 6S
eg 5Q
Hy 7Z
ae 86
CJ SJ
eK 9I
vM In
gn !T
lJ TJ
sH JD
jz ZB
nw Bt
fv tv
wv 7t
vD QB
HB tu
KI 67
Du ?8
BS 70
zG 87
GO 7d
kA LD
uD df
DK fT
OW TS
MS NF
W~ 01
49 1U
9T UN
TM
"#,
      ),
    };
    let score = score_game(&game_9695070671);
    assert!(matches!(score, Ok(_)), "score_game returned {:?}", score);
    let score = score.unwrap();
    assert_eq!(
      Some(&1),
      score.scores.get(&Piece::new(PieceType::Pawn, Color::White, File::B))
    );
    assert_eq!(
      Some(&1),
      score.scores.get(&Piece::new(PieceType::Pawn, Color::White, File::D))
    );
    assert_eq!(
      Some(&1),
      score.scores.get(&Piece::new(PieceType::Pawn, Color::White, File::E))
    );
    assert_eq!(
      Some(&1),
      score.scores.get(&Piece::new(PieceType::Bishop, Color::White, File::F))
    );
    assert_eq!(
      Some(&3),
      score.scores.get(&Piece::new(PieceType::Knight, Color::White, File::B))
    );
    assert_eq!(
      Some(&6),
      score.scores.get(&Piece::new(PieceType::Knight, Color::White, File::G))
    );
    assert_eq!(
      Some(&1),
      score.scores.get(&Piece::new(PieceType::Rook, Color::White, File::A))
    );
    assert_eq!(
      Some(&3),
      score.scores.get(&Piece::new(PieceType::Queen, Color::White, File::D))
    );
    assert_eq!(
      Some(&21),
      score.scores.get(&Piece::new(PieceType::King, Color::White, File::E))
    );

    assert_eq!(
      Some(&1),
      score.scores.get(&Piece::new(PieceType::Bishop, Color::Black, File::C))
    );
    assert_eq!(
      Some(&1),
      score.scores.get(&Piece::new(PieceType::Bishop, Color::Black, File::F))
    );
    assert_eq!(
      Some(&1),
      score.scores.get(&Piece::new(PieceType::Knight, Color::Black, File::B))
    );
    assert_eq!(
      Some(&12),
      score.scores.get(&Piece::new(PieceType::Knight, Color::Black, File::G))
    );
    assert_eq!(
      Some(&3),
      score.scores.get(&Piece::new(PieceType::Rook, Color::Black, File::H))
    );
    assert_eq!(
      Some(&5),
      score.scores.get(&Piece::new(PieceType::Queen, Color::Black, File::D))
    );
    assert_eq!(
      Some(&0),
      score.scores.get(&Piece::new(PieceType::King, Color::Black, File::E))
    );
  }
}
