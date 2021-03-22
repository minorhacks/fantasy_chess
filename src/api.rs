#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GameResponse {
  pub game: Game,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Game {
  #[serde(rename = "moveList")]
  pub move_list: String,
}
