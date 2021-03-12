use std::fmt::Display;

use crate::types::{Board, Hnfen, Piece};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    column: char,
    rank: u8,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.column, self.rank)
    }
}

impl Position {
    pub fn from_indices(x: usize, y: usize) -> Self {
        Position {
            column: (b'a' + x as u8) as char,
            rank: 11 - y as u8,
        }
    }

    /// Returns (x, y) tuple
    pub fn to_indices(&self) -> (usize, usize) {
        (
            (self.column as u8 - b'a') as usize,
            (11 - self.rank) as usize,
        )
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub from: Position,
    pub to: Position,
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_hnfen())
    }
}

impl Hnfen for Move {
    fn as_hnfen(&self) -> String {
        format!("{}{}", self.from, self.to)
    }

    fn from_hnfen(hnfen: &str) -> Option<Self> {
        let move_re = Regex::new(r"^([a-k])(\d{1,2})([a-k])(\d{1,2})$").unwrap();
        let cap = move_re.captures(hnfen).unwrap();
        Some(Move {
            from: Position {
                column: cap.get(1)?.as_str().chars().next()?,
                rank: cap.get(2)?.as_str().parse().ok()?,
            },
            to: Position {
                column: cap.get(3)?.as_str().chars().next()?,
                rank: cap.get(4)?.as_str().parse().ok()?,
            },
        })
    }
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Difference in (x, y) tuple
    pub fn vector(&self, length: usize) -> (isize, isize) {
        let length = length as isize;
        match self {
            Direction::Up => (-length, 0),
            Direction::Down => (length, 0),
            Direction::Left => (0, -length),
            Direction::Right => (0, length),
        }
    }

    pub fn card() -> [Direction; 4] {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
    }
}

pub fn is_corner(x: usize, y: usize) -> bool {
    matches!((x, y), (0, 0) | (0, 10) | (10, 0) | (10, 10))
}

pub fn is_castle(x: usize, y: usize) -> bool {
    is_corner(x, y) || (x, y) == (5, 5)
}

pub fn in_board(x: isize, y: isize) -> bool {
    (0..11).contains(&x) && (0..11).contains(&y)
}

pub fn possible_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();

    let own_pieces = board.pieces(board.next);
    for own_location in own_pieces.iter() {
        let (curr_x, curr_y) = own_location.to_indices();
        let (curr_x, curr_y) = (curr_x as isize, curr_y as isize);
        let is_king = matches!(board.get(own_location), Some(Piece::King));
        for dir in Direction::card().iter() {
            for length in 1..=10 {
                let (diff_x, diff_y) = dir.vector(length);
                let new_x = curr_x + diff_x;
                let new_y = curr_y + diff_y;
                if !in_board(new_x, new_y) {
                    // Reached the edge of the board in this direction
                    break;
                }
                let (new_x, new_y) = (new_x as usize, new_y as usize);
                if !is_king {
                    if is_corner(new_x, new_y) {
                        // Non-King cannot move onto corner
                        break;
                    }
                    if (new_x, new_y) == (5, 5) {
                        // Non-King cannot move onto center castle
                        // But is allowed to move over!
                        continue;
                    }
                }
                if board.get(&Position::from_indices(new_x, new_y)).is_some() {
                    // Something is in the way
                    break;
                }
                moves.push(Move {
                    from: *own_location,
                    to: Position::from_indices(new_x, new_y),
                })
            }
        }
    }

    moves
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::types::*;

    #[test]
    fn display() {
        assert_eq!(
            format!(
                "{}",
                Position {
                    rank: 11,
                    column: 'a'
                }
            ),
            "a11",
        );
        assert_eq!(
            format!(
                "{}",
                Move {
                    from: Position {
                        rank: 11,
                        column: 'a'
                    },
                    to: Position {
                        rank: 1,
                        column: 'b'
                    }
                }
            ),
            "a11b1",
        );
    }

    #[test]
    fn index_conv() {
        assert_eq!(Position::from_indices(0, 0).to_string(), "a11");
        assert_eq!(Position::from_indices(10, 0).to_string(), "k11");
        assert_eq!(Position::from_indices(10, 10).to_string(), "k1");
        assert_eq!(Position::from_indices(0, 10).to_string(), "a1");
        assert_eq!(Position::from_indices(0, 0).to_indices(), (0, 0));
        assert_eq!(Position::from_indices(10, 10).to_indices(), (10, 10));
        assert_eq!(Position::from_indices(0, 10).to_indices(), (0, 10));
    }

    #[test]
    fn get_moves_for_default() {
        let mut board = Board::default();
        let black_moves = possible_moves(&board);
        board.next = Player::White;
        let white_moves = possible_moves(&board);

        assert_eq!(black_moves.len(), 116);
        assert_eq!(white_moves.len(), 60);

        let board = Board::from_hnfen("11/11/11/11/11/11/11/11/11/11/11").unwrap();
        assert_eq!(possible_moves(&board).len(), 0);

        let board = Board::from_hnfen("h10/11/11/11/11/11/11/11/11/11/11 h").unwrap();
        assert_eq!(possible_moves(&board).len(), 20 - 2); // -2 bc of corners

        let board = Board::from_hnfen("a10/11/11/11/11/11/11/11/11/11/11").unwrap();
        assert_eq!(possible_moves(&board).len(), 20 - 2); // -2 bc of corners

        let board = Board::from_hnfen("11/11/11/11/h10/a10/h10/11/11/11/11").unwrap();
        assert_eq!(possible_moves(&board).len(), 10 - 1); // -1 bc of center castle

        let board = Board::from_hnfen("11/11/11/11/a10/h10/a10/11/11/11/11 h").unwrap();
        assert_eq!(possible_moves(&board).len(), 10 - 1); // -1 bc of center castle

        let board = Board::from_hnfen("11/11/11/11/a10/K10/a10/11/11/11/11 h").unwrap();
        assert_eq!(possible_moves(&board).len(), 10);
    }

    #[test]
    fn moves_hnfen() {
        let ex_move = Move {
            from: Position {
                rank: 11,
                column: 'a',
            },
            to: Position {
                rank: 1,
                column: 'b',
            },
        };
        let ex_move_fen = "a11b1";

        assert_eq!(ex_move.as_hnfen(), ex_move_fen);
        assert_eq!(Move::from_hnfen(ex_move_fen).unwrap(), ex_move);
    }
}
