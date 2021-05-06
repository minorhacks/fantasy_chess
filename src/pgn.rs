use crate::db;
use crate::dumbchess;
use itertools::Itertools;
use minorhacks_chess as chess;

#[derive(Clone)]
pub struct GameScore {
  game: db::Game,
  moves: Vec<db::Move>,
  board: chess::Board,
  dumbboard: dumbchess::Board,

  date: String,
  time: String,

  nonstandard_game: bool,
}

impl GameScore {
  pub fn new() -> GameScore {
    GameScore {
      game: db::Game::empty(),
      moves: Vec::new(),
      board: chess::Board::default(),
      dumbboard: dumbchess::Board::starting(),

      date: String::new(),
      time: String::new(),

      nonstandard_game: false,
    }
  }
}

impl Default for GameScore {
  fn default() -> Self {
    Self::new()
  }
}

impl db::Recordable for GameScore {
  fn game(&self) -> db::Result<db::Game> {
    Ok(self.game.clone())
  }

  fn moves(&self) -> db::Result<Vec<db::Move>> {
    Ok(self.moves.clone())
  }
}

impl pgn_reader::Visitor for GameScore {
  type Result = Option<Self>;

  fn begin_variation(&mut self) -> pgn_reader::Skip {
    pgn_reader::Skip(true)
  }

  fn header(&mut self, key: &[u8], value: pgn_reader::RawHeader<'_>) {
    let value = value
      .decode_utf8()
      .expect("invalid UTF-8 in PGN header value")
      .to_string();

    match std::str::from_utf8(key)
      .expect("invalid UTF-8 in PGN header key")
      .to_lowercase()
      .as_str()
    {
      "white" => {
        self.game.white_player_name = value.clone();
        self.game.white_player_id = value;
      }
      "black" => {
        self.game.black_player_name = value.clone();
        self.game.black_player_id = value;
      }
      "whiteelo" => {
        self.game.white_player_rating = value
          .parse::<i32>()
          .unwrap_or_else(|_| panic!("can't parse rating: {}", value));
      }
      "blackelo" => {
        self.game.black_player_rating = value
          .parse::<i32>()
          .unwrap_or_else(|_| panic!("can't parse rating: {}", value));
      }
      // TODO: lichess provides UTCDate and UTCTime
      "utcdate" => self.date = value,
      "utctime" => self.time = value,
      // TODO: chess.com provides Date and EndTime
      "date" => self.date = value,
      "endtime" => {
        // This strips a non-UTC timezone off the end which is straight-up wrong
        self.time = value.split(' ').take(1).join("");
      }
      "site" => {
        // TODO: These site-specific details don't belong here
        if value.starts_with("https://lichess.org/") {
          self.game.source = String::from("lichess.org");
          self.game.source_id =
            value.strip_prefix("https://lichess.org/").unwrap().to_string();
        } else {
          self.game.source = value.to_lowercase();
        }
      }
      "link" => {
        // TODO: These site-specific details don't belong here
        if let Some(suffix) =
          value.strip_prefix("https://www.chess.com/game/live/")
        {
          self.game.source_id = suffix.to_string();
        }
      }
      "event" => {
        if value.to_lowercase().contains("odds chess") {
          self.nonstandard_game = true;
        }
        if value.to_lowercase().contains("chess960") {
          self.nonstandard_game = true;
        }
      }
      _ => (),
    }
  }

  fn end_headers(&mut self) -> pgn_reader::Skip {
    self.game.id = uuid::Uuid::new_v4().to_string();
    let date_time = format!("{} {}", self.date, self.time);
    self.game.end_time =
      chrono::NaiveDateTime::parse_from_str(&date_time, "%Y.%m.%d %H:%M:%S")
        .unwrap_or_else(|_| panic!("invalid date/time: {}", date_time))
        .timestamp();
    println!(
      "Game between {} and {} on {}. Standard: {}",
      self.game.white_player_name,
      self.game.black_player_name,
      self.game.end_time,
      !self.nonstandard_game,
    );
    pgn_reader::Skip(self.nonstandard_game)
  }

  fn san(&mut self, san_plus: pgn_reader::SanPlus) {
    if let Ok(m) =
      chess::ChessMove::from_san(&self.board, &san_plus.to_string())
    {
      let db_move = self
        .dumbboard
        .make_move(
          &dumbchess_square(m.get_source()),
          &dumbchess_square(m.get_dest()),
          promotion_value(m.get_promotion()),
        )
        .expect("invalid move on dumbchess board");
      let mut old_board = chess::Board::default();
      std::mem::swap(&mut old_board, &mut self.board);
      old_board.make_move(m, &mut self.board);
      self.moves.push(db_move);
    } else {
      panic!("invalid move: {}. Board: {:?}", san_plus, self.board);
    }
  }

  fn end_game(&mut self) -> Self::Result {
    if self.nonstandard_game {
      None
    } else {
      Some(self.clone())
    }
  }
}

fn promotion_value(piece: Option<chess::Piece>) -> Option<i32> {
  piece.map(|p| match p {
    chess::Piece::Bishop => 3,
    chess::Piece::Knight => 3,
    chess::Piece::Rook => 5,
    chess::Piece::Queen => 9,
    p => unreachable!("cannot promote to piece: {}", p),
  })
}

fn dumbchess_square(s: chess::Square) -> dumbchess::Square {
  match s {
    chess::Square::A1 => dumbchess::Square::A1,
    chess::Square::A2 => dumbchess::Square::A2,
    chess::Square::A3 => dumbchess::Square::A3,
    chess::Square::A4 => dumbchess::Square::A4,
    chess::Square::A5 => dumbchess::Square::A5,
    chess::Square::A6 => dumbchess::Square::A6,
    chess::Square::A7 => dumbchess::Square::A7,
    chess::Square::A8 => dumbchess::Square::A8,
    chess::Square::B1 => dumbchess::Square::B1,
    chess::Square::B2 => dumbchess::Square::B2,
    chess::Square::B3 => dumbchess::Square::B3,
    chess::Square::B4 => dumbchess::Square::B4,
    chess::Square::B5 => dumbchess::Square::B5,
    chess::Square::B6 => dumbchess::Square::B6,
    chess::Square::B7 => dumbchess::Square::B7,
    chess::Square::B8 => dumbchess::Square::B8,
    chess::Square::C1 => dumbchess::Square::C1,
    chess::Square::C2 => dumbchess::Square::C2,
    chess::Square::C3 => dumbchess::Square::C3,
    chess::Square::C4 => dumbchess::Square::C4,
    chess::Square::C5 => dumbchess::Square::C5,
    chess::Square::C6 => dumbchess::Square::C6,
    chess::Square::C7 => dumbchess::Square::C7,
    chess::Square::C8 => dumbchess::Square::C8,
    chess::Square::D1 => dumbchess::Square::D1,
    chess::Square::D2 => dumbchess::Square::D2,
    chess::Square::D3 => dumbchess::Square::D3,
    chess::Square::D4 => dumbchess::Square::D4,
    chess::Square::D5 => dumbchess::Square::D5,
    chess::Square::D6 => dumbchess::Square::D6,
    chess::Square::D7 => dumbchess::Square::D7,
    chess::Square::D8 => dumbchess::Square::D8,
    chess::Square::E1 => dumbchess::Square::E1,
    chess::Square::E2 => dumbchess::Square::E2,
    chess::Square::E3 => dumbchess::Square::E3,
    chess::Square::E4 => dumbchess::Square::E4,
    chess::Square::E5 => dumbchess::Square::E5,
    chess::Square::E6 => dumbchess::Square::E6,
    chess::Square::E7 => dumbchess::Square::E7,
    chess::Square::E8 => dumbchess::Square::E8,
    chess::Square::F1 => dumbchess::Square::F1,
    chess::Square::F2 => dumbchess::Square::F2,
    chess::Square::F3 => dumbchess::Square::F3,
    chess::Square::F4 => dumbchess::Square::F4,
    chess::Square::F5 => dumbchess::Square::F5,
    chess::Square::F6 => dumbchess::Square::F6,
    chess::Square::F7 => dumbchess::Square::F7,
    chess::Square::F8 => dumbchess::Square::F8,
    chess::Square::G1 => dumbchess::Square::G1,
    chess::Square::G2 => dumbchess::Square::G2,
    chess::Square::G3 => dumbchess::Square::G3,
    chess::Square::G4 => dumbchess::Square::G4,
    chess::Square::G5 => dumbchess::Square::G5,
    chess::Square::G6 => dumbchess::Square::G6,
    chess::Square::G7 => dumbchess::Square::G7,
    chess::Square::G8 => dumbchess::Square::G8,
    chess::Square::H1 => dumbchess::Square::H1,
    chess::Square::H2 => dumbchess::Square::H2,
    chess::Square::H3 => dumbchess::Square::H3,
    chess::Square::H4 => dumbchess::Square::H4,
    chess::Square::H5 => dumbchess::Square::H5,
    chess::Square::H6 => dumbchess::Square::H6,
    chess::Square::H7 => dumbchess::Square::H7,
    chess::Square::H8 => dumbchess::Square::H8,
    unknown_square => unreachable!("unknown square: {}", unknown_square),
  }
}
