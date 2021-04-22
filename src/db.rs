use crate::chess_com;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
  #[error("failed to translate game")]
  GameTranslation,
  #[error("failed to translate move")]
  MoveTranslation {
    #[from]
    source: chess_com::Error,
  },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Move {
  pub move_num: i32,
  pub color: String,
  pub moved_piece: String,
  pub starting_location: String,
  pub ending_location: String,
  pub captured_piece: String,
  pub capture_score: i32,
}

#[derive(Debug)]
pub struct Game {
  pub source: String,
  pub id: String,
  pub end_time: i64,
  pub white_player_id: String,
  pub white_player_name: String,
  pub white_player_rating: i32,
  pub black_player_id: String,
  pub black_player_name: String,
  pub black_player_rating: i32,
}

pub trait Recordable {
  fn game(&self) -> Result<Game>;
  fn moves(&self) -> Result<Vec<Move>>;
}

impl Move {
  pub fn insert_query(
    self,
    game_id: String,
  ) -> sqlx::query::Query<'static, sqlx::Any, sqlx::any::AnyArguments<'static>>
  {
    sqlx::query("INSERT INTO Moves (game_id, move_num, color,
            moved_piece, starting_location, ending_location, captured_piece, capture_score) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(game_id, move_num, color) DO NOTHING")
            .bind(game_id)
            .bind(self.move_num)
            .bind(self.color)
            .bind(self.moved_piece)
            .bind(self.starting_location)
            .bind(self.ending_location)
            .bind(self.captured_piece)
            .bind(self.capture_score)
  }
}

impl Game {
  pub fn insert_query(
    self,
  ) -> sqlx::query::Query<'static, sqlx::Any, sqlx::any::AnyArguments<'static>>
  {
    sqlx::query(
      "INSERT INTO Games (source, id, end_time,
        white_player_id, white_player_name, white_player_rating,
        black_player_id, black_player_name, black_player_rating) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO NOTHING",
    )
    .bind(self.source)
    .bind(self.id)
    .bind(self.end_time)
    .bind(self.white_player_id)
    .bind(self.white_player_name)
    .bind(self.white_player_rating)
    .bind(self.black_player_id)
    .bind(self.black_player_name)
    .bind(self.black_player_rating)
  }
}
