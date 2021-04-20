use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
  #[error("failed to translate game")]
  GameTranslation,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Move {
  game_id: String,
  move_num: i32,
  color: String,
  moved_piece: String,
  starting_location: String,
  ending_location: String,
  captured_piece: String,
  capture_score: i32,
}

#[derive(Debug)]
pub struct Game {
  pub site: String,
  pub id: String,
  pub start_time: u32,
  pub white_player_id: String,
  pub white_player_name: String,
  pub white_player_rating: i32,
  pub black_player_id: String,
  pub black_player_name: String,
  pub black_player_rating: i32,
}

pub trait Recordable: std::fmt::Debug {
  fn game(&self) -> Result<Game>;
}

impl Move {
  pub fn insert_query(
    self,
  ) -> sqlx::query::Query<
    'static,
    sqlx::Sqlite,
    sqlx::sqlite::SqliteArguments<'static>,
  > {
    sqlx::query("INSERT INTO Moves (game_id, move_num, color,
            moved_piece, starting_location, ending_location, captured_piece, capture_score) 
            VALUES (? ? ? ? ? ? ? ?)")
            .bind(self.game_id)
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
  ) -> sqlx::query::Query<
    'static,
    sqlx::Sqlite,
    sqlx::sqlite::SqliteArguments<'static>,
  > {
    sqlx::query(
      "INSERT INTO Games (site, id, start_time,
        white_player_id, white_player_name, white_player_rating,
        black_player_id, black_player_name, black_player_rating) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(self.site)
    .bind(self.id)
    .bind(self.start_time)
    .bind(self.white_player_id)
    .bind(self.white_player_name)
    .bind(self.white_player_rating)
    .bind(self.black_player_id)
    .bind(self.black_player_name)
    .bind(self.black_player_rating)
  }
}
