extern crate clap;
extern crate lazy_static;
extern crate serde_json;
extern crate thiserror;
extern crate tokio;

use crate::api;
use std::collections::HashMap;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
  #[error("piece not found on encoded square: {0}")]
  PieceNotFound(char),
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
  fn value(&self) -> u32 {
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

#[derive(Hash, Eq, PartialEq, Clone)]
struct Piece {
  piece_type: PieceType,
  color: Color,
  file: File,
}

impl Piece {
  fn new(piece_type: PieceType, color: Color, file: File) -> Piece {
    Piece { piece_type, color, file }
  }

  fn value(&self) -> u32 {
    // TODO: If this piece promotes, we want to return the promoted value
    // rather than the value of this original piece.
    self.piece_type.value()
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

struct Board {
  piece_map: HashMap<char, Piece>,
}

impl Board {
  fn starting() -> Board {
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
    }
  }

  fn move_and_score(
    &mut self,
    start: char,
    end: char,
  ) -> Result<(Piece, u32), Error> {
    // Get the piece at the start location
    let (_loc, moved_piece) =
      self.piece_map.remove_entry(&start).ok_or(Error::PieceNotFound(start))?;
    // Get the piece at the end location; if there is one, the score is the value of the piece
    let score = self
      .piece_map
      .get(&end)
      .map(|captured_piece| captured_piece.value())
      .unwrap_or(0);
    // Move the piece from the start to the end
    // TODO: Handle en passant
    // TODO: Handle promotion

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
    // Return the starting piece along with its score
    Ok((moved_piece, score))
  }
}

#[derive(Debug)]
pub struct PieceScore {
  scores: HashMap<Piece, u32>,
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
  }
}
