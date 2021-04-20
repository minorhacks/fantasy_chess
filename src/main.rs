// Analyzes a single chess match by ID, as retrieved from chess.com
// Usage:
//   fantasy_chess score_game <chess.com ID>
//
// Output:
//
//   White:
//     king - 1
//     queen - 1
//     rook_a - 1
//     rook_h - 1
//     knight_b - 1
//     knight_g - 1
//     bishop_c - 1
//     bishop_f - 1
//     pawn_a - 1
//     pawn_b - 1
//     pawn_c - 1
//     pawn_d - 1
//     pawn_e - 1
//     pawn_f - 1
//     pawn_g - 1
//     pawn_h - 1
//
//   Black:
//     king - 1
//     queen - 1
//     rook_a - 1
//     rook_h - 1
//     knight_b - 1
//     knight_g - 1
//     bishop_c - 1
//     bishop_f - 1
//     pawn_a - 1
//     pawn_b - 1
//     pawn_c - 1
//     pawn_d - 1
//     pawn_e - 1
//     pawn_f - 1
//     pawn_g - 1
//     pawn_h - 1

extern crate anyhow;
extern crate clap;
extern crate futures;
extern crate maplit;
extern crate reqwest;
extern crate serde_json;
extern crate thiserror;
extern crate tokio;

use fantasy_chess::api;
use fantasy_chess::{analysis, chess_com};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Parse arguments
  let matches = clap::App::new("fantasy_chess")
    .version("0.1.0")
    .author("Scott Minor <minor@minorhacks.com>")
    .about("CLI for fantasy-chess related operations")
    .subcommand(
      clap::SubCommand::with_name("ingest")
        .about("pull game(s) and ingest into a database")
        .group(
          clap::ArgGroup::with_name("source")
            .args(&["chess_com_game_id", "lichess_game_id"])
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
          clap::Arg::with_name("lichess_game_id")
            .help("ID of the game on Lichess")
            .long("lichess_game_id")
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
    .subcommand(
      clap::SubCommand::with_name("score_game")
        .about("scores an individual game by chess.com ID")
        .arg(
          clap::Arg::with_name("chess.com game ID")
            .help("ID of the game on chess.com")
            .required(true),
        ),
    )
    .get_matches();

  match matches.subcommand() {
    ("score_game", Some(score_game_args)) => {
      // Fetch chess game by ID from chess.com
      let chess_id = score_game_args.value_of("chess.com ID").unwrap();
      let uri =
        format!("https://www.chess.com/callback/live/game/{}", chess_id);
      let body = reqwest::get(&uri).await?.text().await?;
      let res: api::GameResponse = serde_json::from_str(&body)?;
      // API documentation:
      // https://github.com/andyruwruw/chess-web-api/blob/master/documentation/GAME.md
      // moveList parsing:
      // https://github.com/andyruwruw/chess-web-api/issues/10#issuecomment-779735204
      // Calculate which pieces capture how many points
      // Promotion info:
      // https://github.com/andyruwruw/chess-web-api/issues/11#issuecomment-783687021
      let score = analysis::score_game(&res.game)?;
      // Print output
      println!("{}", score);
    }
    ("ingest", Some(ingest_args)) => {
      // Open database
      let db = connect_to_db(ingest_args).await?;

      // Parse game
      let game = parse_game(ingest_args).await?;
      let db_game = game.game()?;
      let game_id = db_game.id.clone();

      // Insert db::Game into DB
      let q = db_game.insert_query().execute(&db).await?;
      println!("query result: {:?}", q);

      // For each move
      let db_moves = game.moves(&game_id)?;
      for m in db_moves {
        // Insert db::Move into DB
        m.insert_query().execute(&db).await?;
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
  // If database arg is sqlite
  if let Some(db_path) = args.value_of("sqlite_db_file") {
    // Open connection to sqlite file
    let connection_string = "sqlite://".to_owned() + db_path;
    //let pool = sqlx::sqlite::SqlitePoolOptions::new()
    let pool = sqlx::any::AnyPoolOptions::new()
      .max_connections(5)
      .connect(&connection_string)
      .await?;
    return Ok(pool);
  } else if let Some(_connection_string) = args.value_of("mysql_db") {
    // If database arg is mysql
    //   Open connection to MySQL DB
    unimplemented!("mysql support not yet implemented")
  } else {
    unimplemented!("unsupported database type")
  }
}

async fn parse_game(
  args: &clap::ArgMatches<'_>,
) -> anyhow::Result<Box<dyn fantasy_chess::db::Recordable>> {
  // If game ID arg is chess.com ID
  if let Some(chess_com_id) = args.value_of("chess_com_game_id") {
    // Parse game info to db::Game
    let uri =
      format!("https://www.chess.com/callback/live/game/{}", chess_com_id);
    let body = reqwest::get(&uri).await?.text().await?;
    let res: chess_com::GameResponse = serde_json::from_str(&body)?;
    return Ok(Box::new(res));
    // Parse moves to db::Moves iter
  }
  // If game ID arg is lichess ID
  //   Parse game info to db::Game
  //   Parse moves to db::Moves iter
  todo!()
}
