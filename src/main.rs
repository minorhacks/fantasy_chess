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
extern crate thiserror;
extern crate tokio;

use std::collections::HashMap;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
enum Error {
    #[error("piece not found on encoded square: {0}")]
    PieceNotFound(char),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GameResponse {
    game: Game,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Game {
    #[serde(rename = "moveList")]
    move_list: String,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Color {
    White,
    Black,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct Piece {
    piece_type: PieceType,
    color: Color,
    file: File,
    value: u32,
}

impl Piece {
    fn new(piece_type: PieceType, color: Color, file: File, value: u32) -> Piece {
        Piece {
            piece_type,
            color,
            file,
            value,
        }
    }
}

impl std::fmt::Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.color, self.piece_type)?;
        match self.piece_type {
            PieceType::Bishop | PieceType::Knight | PieceType::Rook | PieceType::Pawn => {
                write!(f, " {:?}", self.file)?
            }
            _ => (),
        }
        Ok(())
    }
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
                'a' => Piece::new(PieceType::Rook, Color::White, File::A, 5),
                'b' => Piece::new(PieceType::Knight, Color::White, File::B, 3),
                'c' => Piece::new(PieceType::Bishop, Color::White, File::C, 3),
                'd' => Piece::new(PieceType::Queen, Color::White, File::D, 9),
                'e' => Piece::new(PieceType::King, Color::White, File::E, 0),
                'f' => Piece::new(PieceType::Bishop, Color::White, File::F, 3),
                'g' => Piece::new(PieceType::Knight, Color::White, File::G, 3),
                'h' => Piece::new(PieceType::Rook, Color::White, File::H, 5),
                'i' => Piece::new(PieceType::Pawn, Color::White, File::A, 1),
                'j' => Piece::new(PieceType::Pawn, Color::White, File::B, 1),
                'k' => Piece::new(PieceType::Pawn, Color::White, File::C, 1),
                'l' => Piece::new(PieceType::Pawn, Color::White, File::D, 1),
                'm' => Piece::new(PieceType::Pawn, Color::White, File::E, 1),
                'n' => Piece::new(PieceType::Pawn, Color::White, File::F, 1),
                'o' => Piece::new(PieceType::Pawn, Color::White, File::G, 1),
                'p' => Piece::new(PieceType::Pawn, Color::White, File::H, 1),

                '4' => Piece::new(PieceType::Rook, Color::Black, File::A, 5),
                '5' => Piece::new(PieceType::Knight, Color::Black, File::B, 3),
                '6' => Piece::new(PieceType::Bishop, Color::Black, File::C, 3),
                '7' => Piece::new(PieceType::Queen, Color::Black, File::D, 9),
                '8' => Piece::new(PieceType::King, Color::Black, File::E, 0),
                '9' => Piece::new(PieceType::Bishop, Color::Black, File::F, 3),
                '!' => Piece::new(PieceType::Knight, Color::Black, File::G, 3),
                '?' => Piece::new(PieceType::Rook, Color::Black, File::H, 5),
                'W' => Piece::new(PieceType::Pawn, Color::Black, File::A, 1),
                'X' => Piece::new(PieceType::Pawn, Color::Black, File::B, 1),
                'Y' => Piece::new(PieceType::Pawn, Color::Black, File::C, 1),
                'Z' => Piece::new(PieceType::Pawn, Color::Black, File::D, 1),
                '0' => Piece::new(PieceType::Pawn, Color::Black, File::E, 1),
                '1' => Piece::new(PieceType::Pawn, Color::Black, File::F, 1),
                '2' => Piece::new(PieceType::Pawn, Color::Black, File::G, 1),
                '3' => Piece::new(PieceType::Pawn, Color::Black, File::H, 1),
            },
        }
    }

    fn move_and_score(
        &mut self,
        start: char,
        end: char,
    ) -> Result<(Piece, u32), Box<dyn std::error::Error>> {
        // Get the piece at the start location
        let (_loc, moved_piece) = self
            .piece_map
            .remove_entry(&start)
            .ok_or(Error::PieceNotFound(start))?;
        // Get the piece at the end location; if there is one, the score is the value of the piece
        let score = self
            .piece_map
            .get(&end)
            .map(|captured_piece| captured_piece.value)
            .unwrap_or(0);
        // Move the piece from the start to the end
        // TODO: Handle en passant
        match (moved_piece.clone(), start, end) {
            // White kingside castle
            (
                Piece {
                    piece_type: PieceType::King,
                    ..
                },
                'e',
                'g',
            ) => {
                self.piece_map.insert(end, moved_piece.clone());
                let (_, rook) = self
                    .piece_map
                    .remove_entry(&'h')
                    .ok_or(Error::PieceNotFound('h'))?;
                self.piece_map.insert('f', rook);
            }
            // White queenside castle
            (
                Piece {
                    piece_type: PieceType::King,
                    ..
                },
                'e',
                'c',
            ) => {
                self.piece_map.insert(end, moved_piece.clone());
                let (_, rook) = self
                    .piece_map
                    .remove_entry(&'a')
                    .ok_or(Error::PieceNotFound('a'))?;
                self.piece_map.insert('d', rook);
            }
            // Black kingside castle
            (
                Piece {
                    piece_type: PieceType::King,
                    ..
                },
                '8',
                '!',
            ) => {
                self.piece_map.insert(end, moved_piece.clone());
                let (_, rook) = self
                    .piece_map
                    .remove_entry(&'?')
                    .ok_or(Error::PieceNotFound('?'))?;
                self.piece_map.insert('9', rook);
            }
            // Black queenside castle
            (
                Piece {
                    piece_type: PieceType::King,
                    ..
                },
                '8',
                '6',
            ) => {
                self.piece_map.insert(end, moved_piece.clone());
                let (_, rook) = self
                    .piece_map
                    .remove_entry(&'4')
                    .ok_or(Error::PieceNotFound('4'))?;
                self.piece_map.insert('7', rook);
            }
            _ => {
                self.piece_map.insert(end, moved_piece.clone());
            }
        }
        // Return the starting piece along with its score
        Ok((moved_piece, score))
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
