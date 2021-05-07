use std::{collections::VecDeque, sync::Arc};

use tokio::{
  sync::mpsc::{self, Receiver, Sender},
  task::JoinHandle,
};

use async_stream::stream;
use fantasy_chess::pgn;
use futures::{future::join_all, pin_mut, Stream, StreamExt};

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
        )
        .arg(
          clap::Arg::with_name("num_db_connections")
            .help("Number of concurrent database connections")
            .long("num_db_connections")
            .takes_value(true)
            .default_value("10")
            .validator(|s| {
              s.parse::<u32>().map(|_| ()).map_err(|e| e.to_string())
            }),
        )
        .arg(
          clap::Arg::with_name("num_insert_workers")
            .help("Number of concurrent DB insert tasks")
            .long("num_insert_workers")
            .takes_value(true)
            .default_value("10")
            .validator(|s| {
              s.parse::<u32>().map(|_| ()).map_err(|e| e.to_string())
            }),
        ),
    )
    .get_matches();

  match matches.subcommand() {
    ("ingest", Some(ingest_args)) => {
      let db: Arc<_> = connect_to_db(ingest_args).await?;

      if let Some(pgn_filename) = ingest_args.value_of("pgn_file") {
        let f = std::fs::File::open(pgn_filename)?;

        let (completed_queries_tx, mut completed_queries_rx): (
          Sender<usize>,
          Receiver<usize>,
        ) = mpsc::channel(1);

        let mut tasks = Vec::new();
        let mut query_workers_tx = VecDeque::new();

        for _i in 0..ingest_args
          .value_of("num_insert_workers")
          .map(|v| v.parse::<u32>().unwrap())
          .unwrap()
        {
          let (task, tx_chan) =
            start_query_executor(db.clone(), completed_queries_tx.clone());
          tasks.push(task);
          query_workers_tx.push_back(tx_chan);
        }
        drop(completed_queries_tx);

        tasks.push(tokio::spawn(async move {
          let term = console::Term::stderr();
          let mut total_games: usize = 0;
          let mut total_queries: usize = 0;
          while let Some(query_count) = completed_queries_rx.recv().await {
            total_games += 1;
            total_queries += query_count;
            term.clear_line().unwrap();
            term
              .write_str(&format!(
                "GAMES: {}\tINSERTS: {}",
                total_games, total_queries,
              ))
              .unwrap();
          }
          term.write_line("").unwrap();
        }));

        let games = game_stream(f).map(insert_queries);
        pin_mut!(games);
        while let Some(queries) = games.next().await {
          let tx = query_workers_tx.pop_front().unwrap();
          tx.send(queries).await.unwrap_or_else(|_| {
            panic!("failed to send queries to query runner")
          });
          query_workers_tx.push_back(tx);
        }

        drop(query_workers_tx);
        join_all(tasks).await;
      } else {
        unimplemented!()
      }
    }
    _ => {
      unimplemented!("command not implemented")
    }
  }
  Ok(())
}

type StaticSqlQuery =
  sqlx::query::Query<'static, sqlx::Any, sqlx::any::AnyArguments<'static>>;

fn start_query_executor(
  db: Arc<sqlx::Pool<sqlx::Any>>,
  completed_queries_tx: mpsc::Sender<usize>,
) -> (JoinHandle<()>, mpsc::Sender<Vec<StaticSqlQuery>>) {
  let (parsed_games_tx, mut parsed_games_rx): (
    mpsc::Sender<Vec<StaticSqlQuery>>,
    mpsc::Receiver<Vec<StaticSqlQuery>>,
  ) = mpsc::channel(1);

  let task = tokio::spawn(async move {
    while let Some(queries) = parsed_games_rx.recv().await {
      let num_queries = queries.len();
      for query in queries {
        query.execute(&*db).await.unwrap();
      }
      completed_queries_tx.send(num_queries).await.unwrap();
    }
  });

  (task, parsed_games_tx)
}

async fn connect_to_db(
  args: &clap::ArgMatches<'_>,
) -> sqlx::Result<Arc<sqlx::Pool<sqlx::Any>>> {
  if let Some(db_path) = args.value_of("sqlite_db_file") {
    let connection_string = "sqlite://".to_owned() + db_path;
    let pool = sqlx::any::AnyPoolOptions::new()
      .max_connections(
        args.value_of("num_db_connections").unwrap().parse::<u32>().unwrap(),
      )
      .connect(&connection_string)
      .await?;
    return Ok(Arc::new(pool));
  } else if let Some(connection_string) = args.value_of("mysql_db") {
    let connection_string = "mysql://".to_owned() + connection_string;
    let pool = sqlx::any::AnyPoolOptions::new()
      .max_connections(
        args.value_of("num_db_connections").unwrap().parse::<u32>().unwrap(),
      )
      .connect(&connection_string)
      .await?;
    return Ok(Arc::new(pool));
  } else {
    unimplemented!("unsupported database type")
  }
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
