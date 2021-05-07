extern crate anyhow;
extern crate clap;
extern crate futures;
extern crate itertools;
extern crate maplit;
extern crate reqwest;
extern crate serde_json;
extern crate thiserror;
extern crate tokio;

use std::{convert::TryInto, pin::Pin, sync::Arc};
use tokio::sync::Mutex;

use anyhow::anyhow;
use async_stream::stream;
use fantasy_chess::chess_com;
use fantasy_chess::pgn;
use futures::{future::try_join_all, pin_mut, Stream, StreamExt};

const MAX_DB_CONNECTIONS: u32 = 40;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let matches = clap::App::new("fantasy_chess")
    .version("0.1.0")
    .author("Scott Minor <minor@minorhacks.com>")
    .about("CLI for fantasy-chess related operations")
    .subcommand(
      clap::SubCommand::with_name("ingest")
        .about("pull game(s) and ingest into a database")
        .group(
          clap::ArgGroup::with_name("source")
            .args(&["chess_com_game_id", "pgn_file"])
            .required(true),
        )
        .group(
          clap::ArgGroup::with_name("db")
            .args(&["sqlite_db_file", "mysql_db"])
            .required(true),
        )
        .arg(
          clap::Arg::with_name("chess_com_game_id")
            .help("ID of the game on chess.com")
            .long("chess_com_game_id")
            .takes_value(true),
        )
        .arg(
          clap::Arg::with_name("pgn_file")
            .help("Path to PGN game database")
            .long("pgn_file")
            .takes_value(true),
        )
        .arg(
          clap::Arg::with_name("sqlite_db_file")
            .help(
              "Path to sqlite DB file. Will be created if it doesn't exist.",
            )
            .long("sqlite_db_file")
            .takes_value(true),
        )
        .arg(
          clap::Arg::with_name("mysql_db")
            .help("MySQL DB connection string")
            .long("mysql_db")
            .takes_value(true),
        ),
    )
    .get_matches();

  match matches.subcommand() {
    ("ingest", Some(ingest_args)) => {
      let db: Arc<Mutex<_>> = connect_to_db(ingest_args).await?;

      if let Some(pgn_filename) = ingest_args.value_of("pgn_file") {
        let f = std::fs::File::open(pgn_filename)?;

        let games = game_stream(f);
        let games = games.map(insert_queries);
        pin_mut!(games);
        while let Some(queries) = games.next().await {
          let db = db.clone();
          tokio::spawn(async move {
            for query in queries {
              let guard = db.lock().await;
              query.execute(&*guard).await.unwrap();
            }
          });
        }
      } else {
        unimplemented!()
      }

      //let games = parse_games(ingest_args).await?;
      //for game in games {
      //  let db_game = game.game()?;
      //  let game_id = db_game.id.clone();
      //  let db_moves = game.moves()?;
      //  let mut handles = Vec::new();
      //  let pool_for_game_insert = db.clone();
      //  db_game.insert_query().execute(&*pool_for_game_insert).await.unwrap();
      //  for m in db_moves {
      //    let db = db.clone();
      //    let game_id = game_id.clone();
      //    handles.push(tokio::spawn(async move {
      //      m.insert_query(game_id.clone()).execute(&*db).await.unwrap()
      //    }));
      //  }
      //  try_join_all(handles).await?;
      //  bar.inc(1);
      //}
    }
    _ => {
      unimplemented!("command not implemented")
    }
  }
  Ok(())
}

async fn connect_to_db(
  args: &clap::ArgMatches<'_>,
) -> sqlx::Result<Arc<Mutex<sqlx::Pool<sqlx::Any>>>> {
  if let Some(db_path) = args.value_of("sqlite_db_file") {
    let connection_string = "sqlite://".to_owned() + db_path;
    let pool = sqlx::any::AnyPoolOptions::new()
      .max_connections(MAX_DB_CONNECTIONS)
      .connect(&connection_string)
      .await?;
    return Ok(Arc::new(Mutex::new(pool)));
  } else if let Some(connection_string) = args.value_of("mysql_db") {
    let connection_string = "mysql://".to_owned() + connection_string;
    let pool = sqlx::any::AnyPoolOptions::new()
      .max_connections(MAX_DB_CONNECTIONS)
      .connect(&connection_string)
      .await?;
    return Ok(Arc::new(Mutex::new(pool)));
  } else {
    unimplemented!("unsupported database type")
  }
}

async fn parse_games(
  args: &clap::ArgMatches<'_>,
) -> anyhow::Result<Vec<Box<dyn fantasy_chess::db::Recordable>>> {
  if let Some(chess_com_id) = args.value_of("chess_com_game_id") {
    let uri =
      format!("https://www.chess.com/callback/live/game/{}", chess_com_id);
    let body = reqwest::get(&uri).await?.text().await?;
    let res: chess_com::GameResponse = serde_json::from_str(&body)?;
    return Ok(vec![Box::new(res)]);
  } else if let Some(pgn_filename) = args.value_of("pgn_file") {
    let f = std::fs::File::open(pgn_filename)?;
    let mut scanner = pgn_reader::BufferedReader::new(f);
    let mut games: Vec<Box<dyn fantasy_chess::db::Recordable>> = Vec::new();
    let mut game_counter = 0;
    loop {
      let mut visitor = pgn::GameScore::new();
      let res = scanner.read_game(&mut visitor);
      match res {
        Ok(Some(Some(score))) => games.push(Box::new(score)),
        Ok(Some(None)) => (),
        Ok(None) => return Ok(games),
        Err(e) => {
          return Err(anyhow!("processing game {}: {}", game_counter, e))
        }
      }
      game_counter += 1;
    }
  }
  unimplemented!()
}

fn game_stream<R: std::io::Read>(
  reader: R,
) -> impl Stream<Item = Box<dyn fantasy_chess::db::Recordable>> {
  stream! {
    let mut scanner = pgn_reader::BufferedReader::new(reader);
    loop {
    let mut visitor = pgn::GameScore::new();
    let res = scanner.read_game(&mut visitor).unwrap(); // TODO: Remove unwrap
    match res {
      Some(Some(score)) => {let b: Box<dyn fantasy_chess::db::Recordable> = Box::new(score); yield b;},
      Some(None) => continue,
      None => break,
    };
    }
  }
}

fn insert_queries(
  game: Box<dyn fantasy_chess::db::Recordable>,
) -> Vec<sqlx::query::Query<'static, sqlx::Any, sqlx::any::AnyArguments<'static>>>
{
  let mut inserts = Vec::new();
  let db_game = game.game().unwrap();
  let game_id = db_game.id.clone();
  inserts.push(db_game.insert_query());
  let db_moves = game.moves().unwrap();
  for m in db_moves {
    inserts.push(m.insert_query(game_id.clone()));
  }
  inserts
}
