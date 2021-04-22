use std::mem::swap;

use shakmaty::Position;

use crate::db;

pub struct GameScore {
  game: Option<db::Game>,
  moves: Vec<db::Move>,
  board: shakmaty::Chess,
}

impl GameScore {
  pub fn new() -> GameScore {
    GameScore {
      game: None,
      moves: Vec::new(),
      board: shakmaty::Chess::default(),
    }
  }
}

impl Default for GameScore {
  fn default() -> Self {
    Self::new()
  }
}

impl pgn_reader::Visitor for GameScore {
  type Result = (db::Game, Vec<db::Move>);

  fn begin_variation(&mut self) -> pgn_reader::Skip {
    pgn_reader::Skip(true)
  }

  fn san(&mut self, san_plus: pgn_reader::SanPlus) {
    if let Ok(m) = san_plus.san.to_move(&self.board) {
      // Based on the move, we need to know:
      // * which piece got moved
      // * if it captured, which piece got captured
      self.board.play_unchecked(&m);
    } else {
      panic!("invalid move");
    }
  }

  fn end_game(&mut self) -> Self::Result {
    let mut moves = Vec::new();
    swap(&mut moves, &mut self.moves);
    (self.game.take().unwrap(), moves)
  }
}
