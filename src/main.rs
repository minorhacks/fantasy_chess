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
extern crate maplit;
extern crate reqwest;
extern crate serde_json;
extern crate thiserror;
extern crate tokio;

use fantasy_chess::analysis;
use fantasy_chess::api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Parse arguments
  let matches = clap::App::new("fantasy_chess")
    .version("0.1.0")
    .author("Scott Minor <minor@minorhacks.com>")
    .about("CLI for fantasy-chess related operations")
    .subcommand(
      clap::SubCommand::with_name("score_game")
        .about("scores an individual game by chess.com ID")
        .arg(
          clap::Arg::with_name("chess.com ID")
            .help("ID of the game on chess.com")
            .required(true),
        ),
    )
    .get_matches();

  if let Some(score_game_args) = matches.subcommand_matches("score_game") {
    // Fetch chess game by ID from chess.com
    let chess_id = score_game_args.value_of("chess.com ID").unwrap();
    let uri = format!("https://www.chess.com/callback/live/game/{}", chess_id);
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
  } else {
    unimplemented!("command not implemented");
  }
  Ok(())
}
