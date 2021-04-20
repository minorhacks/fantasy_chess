use crate::analysis;
use crate::db;

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
      source: "chess.com".to_owned(),
      id: self.game.id.to_string(),
      end_time: self.game.end_time,
      white_player_id: white_player.id.to_string(),
      white_player_name: white_player.username.clone(),
      white_player_rating: white_player.rating,
      black_player_id: black_player.id.to_string(),
      black_player_name: black_player.username.clone(),
      black_player_rating: black_player.rating,
    })
  }

  fn moves(&self, game_id: &str) -> db::Result<Vec<db::Move>> {
    let mut board = analysis::Board::starting();
    let mut move_list = self.game.move_list.chars().fuse();
    let mut parsed_moves = Vec::new();

    while let Some(start) = move_list.next() {
      if let Some(end) = move_list.next() {
        let m = board.make_move(game_id, start, end)?;
        parsed_moves.push(m);
      } else {
        return Err(db::Error::GameTranslation);
      }
    }
    Ok(parsed_moves)
  }
}

// =============================================================================
// Junk to help replay games
// =============================================================================
