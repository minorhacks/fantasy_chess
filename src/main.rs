extern crate anyhow;
extern crate clap;
extern crate futures;
extern crate itertools;
extern crate maplit;
extern crate reqwest;
extern crate serde_json;
extern crate thiserror;
extern crate tokio;

use fantasy_chess::chess_com;
use fantasy_chess::pgn;

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
      let db = connect_to_db(ingest_args).await?;

      let game = parse_game(ingest_args).await?;
      let db_game = game.game()?;
      let game_id = db_game.id.clone();

      let q = db_game.insert_query().execute(&db).await?;

      let db_moves = game.moves()?;
      for m in db_moves {
        m.insert_query(game_id.clone()).execute(&db).await?;
      }
    }
    _ => {
      unimplemented!("command not implemented")
    }
  }
  Ok(())
}

async fn connect_to_db(
  args: &clap::ArgMatches<'_>,
) -> sqlx::Result<sqlx::Pool<sqlx::Any>> {
  if let Some(db_path) = args.value_of("sqlite_db_file") {
    let connection_string = "sqlite://".to_owned() + db_path;
    let pool = sqlx::any::AnyPoolOptions::new()
      .max_connections(5)
      .connect(&connection_string)
      .await?;
    return Ok(pool);
  } else if let Some(connection_string) = args.value_of("mysql_db") {
    let connection_string = "mysql://".to_owned() + connection_string;
    let pool = sqlx::any::AnyPoolOptions::new()
      .max_connections(5)
      .connect(&connection_string)
      .await?;
    return Ok(pool);
  } else {
    unimplemented!("unsupported database type")
  }
}

async fn parse_game(
  args: &clap::ArgMatches<'_>,
) -> anyhow::Result<Box<dyn fantasy_chess::db::Recordable>> {
  if let Some(chess_com_id) = args.value_of("chess_com_game_id") {
    let uri =
      format!("https://www.chess.com/callback/live/game/{}", chess_com_id);
    let body = reqwest::get(&uri).await?.text().await?;
    let res: chess_com::GameResponse = serde_json::from_str(&body)?;
    return Ok(Box::new(res));
  } else if let Some(pgn_filename) = args.value_of("pgn_file") {
    let f = std::fs::File::open(pgn_filename)?;
    let scanner = pgn::PgnSplitter::new(f);
    //for (i, f) in scanner.enumerate() {
    //  println!("+++++ PGN {} +++++\n{}", i, f);
    //}
    println!("PGN count: {}", scanner.count());
  }
  unimplemented!()
}
