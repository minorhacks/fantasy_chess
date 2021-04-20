use itertools::Itertools;
use std::io::BufRead;

pub struct PgnSplitter<T: std::io::Read> {
  rdr: std::iter::Peekable<std::io::Lines<std::io::BufReader<T>>>,
  done: bool,
}

impl<T: std::io::Read> PgnSplitter<T> {
  pub fn new(rdr: T) -> PgnSplitter<T> {
    PgnSplitter {
      rdr: std::io::BufReader::new(rdr).lines().peekable(),
      done: false,
    }
  }
}

impl<T: std::io::Read> Iterator for PgnSplitter<T> {
  type Item = String;

  fn next(&mut self) -> Option<Self::Item> {
    let mut done = self.done;
    if done {
      return None;
    }
    if self.rdr.peek().is_none() {
      self.done = true;
      return None;
    }
    let mut empty_line_count = 0;
    let pgn = self
      .rdr
      .fold_while(String::new(), |acc, line| match line {
        Err(_e) => {
          done = true;
          itertools::FoldWhile::Done(acc)
        }
        Ok(s) => {
          if s.is_empty() {
            empty_line_count += 1;
          }
          let new_acc = acc + "\n" + &s;
          if empty_line_count == 2 {
            itertools::FoldWhile::Done(new_acc)
          } else {
            itertools::FoldWhile::Continue(new_acc)
          }
        }
      })
      .into_inner();
    self.done = done;
    if done {
      None
    } else {
      Some(pgn)
    }
  }
}
