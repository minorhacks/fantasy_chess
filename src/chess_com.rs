use std::collections::HashMap;

use crate::db;
use crate::dumbchess::{Board, Square};

// =============================================================================
// API Types
// =============================================================================

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GameResponse {
  pub game: Game,
  pub players: Players,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Game {
  id: u64,
  #[serde(rename = "endTime")]
  pub end_time: i64,
  #[serde(rename = "moveList")]
  pub move_list: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Players {
  top: Player,
  bottom: Player,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Player {
  id: u64,
  username: String,
  color: String,
  rating: i32,
}

impl db::Recordable for GameResponse {
  fn game(&self) -> db::Result<db::Game> {
    let (white_player, black_player) = match self.players.top.color.as_str() {
      "white" => (&self.players.top, &self.players.bottom),
      "black" => (&self.players.bottom, &self.players.top),
      _ => return Err(db::Error::GameTranslation),
    };
    Ok(db::Game {
      id: uuid::Uuid::new_v4().to_string(),
      source: "chess.com".to_owned(),
      source_id: self.game.id.to_string(),
      end_time: self.game.end_time,
      white_player_id: white_player.id.to_string(),
      white_player_name: white_player.username.clone(),
      white_player_rating: white_player.rating,
      black_player_id: black_player.id.to_string(),
      black_player_name: black_player.username.clone(),
      black_player_rating: black_player.rating,
    })
  }

  fn moves(&self) -> db::Result<Vec<db::Move>> {
    let mut board = Board::starting();
    let mut move_list = self.game.move_list.chars().fuse();
    let mut parsed_moves = Vec::new();

    while let Some(start) = move_list.next() {
      if let Some(end) = move_list.next() {
        let mut promotion = None;
        let start_square = start.into();
        // Promotion is handled here by looking at the end move; if it is a
        // promotion move, it will have a special character that doesn't correspond
        // to any square on the board. From this char, we can deduce:
        // * the promotion square, which is calculated based on the promotion
        //   direction and the starting square
        // * the piece type that this piece is promoted to
        let end_square = if PROMOTION_DIR.contains_key(&end) {
          promotion = Some(*PROMOTION_VALUE.get(&end).unwrap());
          PROMOTION_DIR.get(&end).unwrap().get(start_square).unwrap()
        } else {
          end.into()
        };
        let m = board.make_move(start_square, end_square, promotion)?;
        parsed_moves.push(m);
      } else {
        return Err(db::Error::GameTranslation);
      }
    }
    Ok(parsed_moves)
  }
}

lazy_static! {
  static ref PROMOTE_LEFT: HashMap<Square, Square> = maplit::hashmap! {
    Square::B2 => Square::A1,
    Square::C2 => Square::B1,
    Square::D2 => Square::C1,
    Square::E2 => Square::D1,
    Square::F2 => Square::E1,
    Square::G2 => Square::F1,
    Square::H2 => Square::G1,
    Square::B7 => Square::A8,
    Square::C7 => Square::B8,
    Square::D7 => Square::C8,
    Square::E7 => Square::D8,
    Square::F7 => Square::E8,
    Square::G7 => Square::F8,
    Square::H7 => Square::G8,
  };
  static ref PROMOTE_STRAIGHT: HashMap<Square, Square> = maplit::hashmap! {
    Square::A2 => Square::A1,
    Square::B2 => Square::B1,
    Square::C2 => Square::C1,
    Square::D2 => Square::D1,
    Square::E2 => Square::E1,
    Square::F2 => Square::F1,
    Square::G2 => Square::G1,
    Square::H2 => Square::H1,
    Square::A7 => Square::A8,
    Square::B7 => Square::B8,
    Square::C7 => Square::C8,
    Square::D7 => Square::D8,
    Square::E7 => Square::E8,
    Square::F7 => Square::F8,
    Square::G7 => Square::G8,
    Square::H7 => Square::H8,
  };
  static ref PROMOTE_RIGHT: HashMap<Square, Square> = maplit::hashmap! {
    Square::A2 => Square::B1,
    Square::B2 => Square::C1,
    Square::C2 => Square::D1,
    Square::D2 => Square::E1,
    Square::E2 => Square::F1,
    Square::F2 => Square::G1,
    Square::G2 => Square::H1,
    Square::A7 => Square::B8,
    Square::B7 => Square::C8,
    Square::C7 => Square::D8,
    Square::D7 => Square::E8,
    Square::E7 => Square::F8,
    Square::F7 => Square::G8,
    Square::G7 => Square::H8,
  };
  static ref PROMOTION_DIR: HashMap<char, &'static HashMap<Square, Square>> = maplit::hashmap! {
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
}
