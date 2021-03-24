#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GameResponse {
  pub game: Game,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Game {
  #[serde(rename = "moveList")]
  pub move_list: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GameSummary {
  pub id: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PlayerResponse {
  pub player_id: u64,
}

#[cfg(test)]
mod tests {
  use super::*;

  fn testdata_to_string(path: &str) -> String {
    std::fs::read_to_string(format!(
      "{}/src/testdata/{}",
      env!("CARGO_MANIFEST_DIR"),
      path,
    ))
    .unwrap_or_else(|_| panic!("can't read input testdata file: {}", path))
  }

  #[test]
  fn test_parse_player_response() {
    let data = testdata_to_string("player_response.json");
    let parsed: Result<PlayerResponse, _> = serde_json::from_str(&data);
    assert!(matches!(parsed, Ok(_)));
    assert_eq!(31513926, parsed.unwrap().player_id);
  }

  #[test]
  fn test_parse_game_response() {
    let data = testdata_to_string("game_response.json");
    let parsed: Result<GameResponse, _> = serde_json::from_str(&data);
    assert!(matches!(parsed, Ok(_)));
    assert_eq!("mC0KlB3NBJYIJQXHfHWGgvZJbs1LcD2Mdl6Seg5QHy7Zae86CJSJeK9IvMIngn!TlJTJsHJDjzZBnwBtfvtvwv7tvDQBHBtuKI67Du?8BS70zG87GO7dkALDuDdfDKfTOWTSMSNFW~01491U9TUNTM", parsed.unwrap().game.move_list);
  }

  #[test]
  fn test_parse_game_summary_list() {
    let data = testdata_to_string("games_for_player.json");
    let parsed: Result<Vec<GameSummary>, _> = serde_json::from_str(&data);
    assert!(matches!(parsed, Ok(_)));
    assert_eq!(20, parsed.unwrap().len());
  }
}
