#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GameResponse {
  pub game: Game,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Game {
  id: String,

  #[serde(rename = "moveList")]
  pub move_list: String,
}
