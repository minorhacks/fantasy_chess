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

extern crate clap;
#[macro_use]
extern crate maplit;
extern crate reqwest;
extern crate serde_json;
extern crate tokio;

use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GameResponse {
    game: Game,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Game {
    #[serde(rename = "moveList")]
    move_list: String,
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum Piece {
    WhiteKing,
    WhiteQueen,
    WhiteRookA,
    WhiteRookH,
    WhiteKnightB,
    WhiteKnightG,
    WhiteBishopC,
    WhiteBishopF,
    WhitePawnA,
    WhitePawnB,
    WhitePawnC,
    WhitePawnD,
    WhitePawnE,
    WhitePawnF,
    WhitePawnG,
    WhitePawnH,
    BlackKing,
    BlackQueen,
    BlackRookA,
    BlackRookH,
    BlackKnightB,
    BlackKnightG,
    BlackBishopC,
    BlackBishopF,
    BlackPawnA,
    BlackPawnB,
    BlackPawnC,
    BlackPawnD,
    BlackPawnE,
    BlackPawnF,
    BlackPawnG,
    BlackPawnH,
}

struct Board {
    piece_map: HashMap<char, Piece>,
}

#[derive(Debug)]
struct PieceScore {
    scores: HashMap<Piece, u32>,
}

impl Board {
    fn starting() -> Board {
        Board {
            piece_map: maplit::hashmap! {
                'a' => Piece::WhiteRookA,
                'b' => Piece::WhiteKnightB,
                'c' => Piece::WhiteBishopC,
                'd' => Piece::WhiteQueen,
                'e' => Piece::WhiteKing,
                'f' => Piece::WhiteBishopF,
                'g' => Piece::WhiteKnightG,
                'h' => Piece::WhiteRookH,
                'i' => Piece::WhitePawnA,
                'j' => Piece::WhitePawnB,
                'k' => Piece::WhitePawnC,
                'l' => Piece::WhitePawnD,
                'm' => Piece::WhitePawnE,
                'n' => Piece::WhitePawnF,
                'o' => Piece::WhitePawnG,
                'p' => Piece::WhitePawnH,
                '4' => Piece::BlackRookA,
                '5' => Piece::BlackKnightB,
                '6' => Piece::BlackBishopC,
                '7' => Piece::BlackQueen,
                '8' => Piece::BlackKing,
                '9' => Piece::BlackBishopF,
                '!' => Piece::BlackKnightG,
                '?' => Piece::BlackRookH,
                'W' => Piece::BlackPawnA,
                'X' => Piece::BlackPawnB,
                'Y' => Piece::BlackPawnC,
                'Z' => Piece::BlackPawnD,
                '0' => Piece::BlackPawnE,
                '1' => Piece::BlackPawnF,
                '2' => Piece::BlackPawnG,
                '3' => Piece::BlackPawnH,
            },
        }
    }

    fn move_and_score(
        &mut self,
        start: char,
        end: char,
    ) -> Result<(Piece, u32), Box<dyn std::error::Error>> {
        todo!();
    }
}

fn score_game(game: &Game) -> Result<PieceScore, Box<dyn std::error::Error>> {
    let mut final_score = PieceScore {
        scores: HashMap::new(),
    };
    let mut board = Board::starting();
    let moves = &game.move_list.clone();
    let mut move_list = moves.chars().fuse();

    while let Some(start) = move_list.next() {
        if let Some(end) = move_list.next() {
            let (piece, score) = board.move_and_score(start, end)?;
            *final_score.scores.entry(piece).or_insert(0) += score;
        } else {
            panic!();
        }
    }
    Ok(final_score)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        let res: GameResponse = serde_json::from_str(&body)?;
        println!("{:?}", res);
        // API documentation:
        // https://github.com/andyruwruw/chess-web-api/blob/master/documentation/GAME.md
        // moveList parsing:
        // https://github.com/andyruwruw/chess-web-api/issues/10#issuecomment-779735204
        // Calculate which pieces capture how many points
        let score = score_game(&res.game)?;
        // Print output
        println!("{:?}", score);
    } else {
        unimplemented!("command not implemented");
    }
    println!("Hello, world!");
    Ok(())
}
