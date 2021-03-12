use std::convert::TryInto;

use crate::moves::{in_board, is_castle, is_corner, Direction, Move, Position};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Board {
    pub ranks: [Rank; 11],
    pub next: Player,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rank {
    pub fields: [Option<Piece>; 11],
}

const WHITE: &str = "h";
const BLACK: &str = "a";
const KING: &str = "K";
const RANK_SEP: &str = "/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Piece {
    Normal(Player),
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Player {
    /// Starts, is attacker
    Black,
    /// Second, is defender
    White,
}

pub trait Hnfen: Sized {
    fn as_hnfen(&self) -> String;
    fn from_hnfen(hnfen: &str) -> Option<Self>;
}

impl Hnfen for Player {
    fn as_hnfen(&self) -> String {
        match self {
            Player::Black => BLACK,
            Player::White => WHITE,
        }
        .to_string()
    }

    fn from_hnfen(hnfen: &str) -> Option<Self> {
        match hnfen {
            BLACK => Some(Player::Black),
            WHITE => Some(Player::White),
            _ => None,
        }
    }
}

impl Player {
    pub fn opposite(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

impl Hnfen for Piece {
    fn as_hnfen(&self) -> String {
        match self {
            Piece::Normal(Player::Black) => BLACK,
            Piece::Normal(Player::White) => WHITE,
            Piece::King => KING,
        }
        .to_owned()
    }

    fn from_hnfen(hnfen: &str) -> Option<Self> {
        Some(match hnfen {
            BLACK => Piece::Normal(Player::Black),
            WHITE => Piece::Normal(Player::White),
            KING => Piece::King,
            _ => return None,
        })
    }
}

impl Piece {
    pub fn color(&self) -> Player {
        match self {
            Piece::Normal(c) => *c,
            Piece::King => Player::White,
        }
    }
}

impl Hnfen for Rank {
    fn as_hnfen(&self) -> String {
        let mut empty_prec = 0;
        let mut buf = String::new();
        for k in self.fields.iter() {
            if let Some(p) = k {
                if empty_prec > 0 {
                    buf.push_str(&format!("{}", empty_prec));
                    empty_prec = 0;
                }
                buf.push_str(&p.as_hnfen());
            } else {
                empty_prec += 1;
            }
        }
        if empty_prec > 0 {
            buf.push_str(&format!("{}", empty_prec));
        }
        buf
    }

    fn from_hnfen(hnfen: &str) -> Option<Self> {
        // NOTE this is when I realized that using multi-digit numbers makes the language context-sensitive.
        let mut rank = Rank { fields: [None; 11] };

        enum C {
            Number(usize),
            Character(Piece),
        }

        let mut groups: Vec<C> = Vec::new();

        for k in hnfen.chars() {
            match k {
                _ if k.is_numeric() => {
                    if let Some(C::Number(c)) = groups.last_mut() {
                        *c = *c * 10 + k.to_digit(10).unwrap() as usize
                    } else {
                        groups.push(C::Number(k.to_digit(10).unwrap() as usize))
                    }
                }
                _ => groups.push(C::Character(Piece::from_hnfen(&k.to_string())?)),
            };
        }

        let mut c_index = 0;
        for group in groups.into_iter() {
            match group {
                C::Number(k) => {
                    c_index += k;
                }
                C::Character(p) => {
                    rank.fields[c_index] = Some(p);
                    c_index += 1;
                }
            }
        }

        if c_index != 11 {
            None
        } else {
            Some(rank)
        }
    }
}

impl Default for Rank {
    fn default() -> Self {
        Rank { fields: [None; 11] }
    }
}

impl Rank {
    pub fn pretty(&self) -> String {
        let mut buf = String::new();
        for f in self.fields.iter() {
            match f {
                Some(p) => buf.push_str(&p.as_hnfen()),
                None => buf.push(' '),
            }
        }
        buf
    }
}

impl Board {
    pub fn get(&self, pos: &Position) -> Option<Piece> {
        let (x, y) = pos.to_indices();
        self.ranks[y].fields[x]
    }

    pub fn set(&mut self, pos: &Position, piece: &Option<Piece>) {
        let (x, y) = pos.to_indices();
        self.ranks[y].fields[x] = *piece;
    }

    pub fn pieces(&self, color: Player) -> Vec<Position> {
        let mut pos = Vec::new();
        for (y, rank) in self.ranks.iter().enumerate() {
            for (x, piece) in rank.fields.iter().enumerate() {
                match piece {
                    Some(Piece::Normal(c)) if *c == color => pos.push(Position::from_indices(x, y)),
                    Some(Piece::King) if color == Player::White => {
                        pos.push(Position::from_indices(x, y))
                    }
                    _ => {}
                }
            }
        }
        pos
    }

    pub fn king(&self) -> Option<Position> {
        for (y, rank) in self.ranks.iter().enumerate() {
            for (x, piece) in rank.fields.iter().enumerate() {
                if let Some(Piece::King) = piece {
                    return Some(Position::from_indices(x, y));
                }
            }
        }
        None
    }

    pub fn king_escaped(&self) -> bool {
        if let Some(pos) = self.king() {
            let (x, y) = pos.to_indices();
            is_corner(x, y)
        } else {
            false
        }
    }

    /// Returns true if a king at position pos *would* be captured
    pub fn is_king_capture(&self, pos: &Position) -> bool {
        //println!("Potential king capture with board\n{}", self.pretty());

        let pos = pos.to_indices();
        for dir in Direction::card().iter() {
            let dir_diff = dir.vector(1);
            let check_place = (pos.0 as isize + dir_diff.0, pos.1 as isize + dir_diff.1);

            if !in_board(check_place.0, check_place.1) {
                return false; // not captured
            }

            let check_place = (check_place.0 as usize, check_place.1 as usize);
            if is_castle(check_place.0, check_place.1) {
                continue; // captured in this direction
            }

            if let Some(p) = self.get(&Position::from_indices(check_place.0, check_place.1)) {
                if p.color() == Player::White {
                    return false; // not captured
                } else {
                    continue; // captured in this direction
                }
            } else {
                return false; // not captured
            }
        }

        true
    }

    pub fn apply(&mut self, mov: &Move) {
        let (x, y) = mov.from.to_indices();
        let piece = if let Some(p) = self.ranks[y].fields[x] {
            p
        } else {
            // Probably a nop move
            return;
        };
        let move_color = piece.color();
        self.ranks[y].fields[x] = None;
        let (x, y) = mov.to.to_indices();
        self.ranks[y].fields[x] = Some(piece);

        for dir in Direction::card().iter() {
            let dir_diff = dir.vector(1);
            let check_place = (x as isize + dir_diff.0, y as isize + dir_diff.1);
            if !in_board(check_place.0, check_place.1) {
                continue;
            }
            let other_place = (check_place.0 as usize, check_place.1 as usize);
            let other_is_king =
                match self.get(&Position::from_indices(other_place.0, other_place.1)) {
                    Some(Piece::Normal(c)) if c != move_color => {
                        // Potential take of other_piece
                        false
                    }
                    Some(Piece::King) if move_color == Player::Black => {
                        // Potential take of king!
                        true
                    }
                    _ => {
                        // Nothing here to take, continue with next direction
                        continue;
                    }
                };
            let opposite_place = (
                other_place.0 as isize + dir_diff.0,
                other_place.1 as isize + dir_diff.1,
            );
            if !in_board(opposite_place.0, opposite_place.1) {
                continue;
            }
            let opposite_place = (opposite_place.0 as usize, opposite_place.1 as usize);

            if other_is_king {
                if self.is_king_capture(&Position::from_indices(other_place.0, other_place.1)) {
                    // Took the king, that's pretty cool
                    self.set(&Position::from_indices(other_place.0, other_place.1), &None);
                } else {
                    // Not taking the king
                    continue;
                }
            } else if let Some(p) =
                self.get(&Position::from_indices(opposite_place.0, opposite_place.1))
            {
                // Is surrounded by other piece of move_color
                if p.color() == move_color {
                    self.set(&Position::from_indices(other_place.0, other_place.1), &None);
                }
            }
        }
        self.next = move_color.opposite();
    }

    pub fn pretty(&self) -> String {
        let mut pp = "╔═══════════╗\n".to_string();
        pp.push_str(
            &self
                .ranks
                .iter()
                .map(|r| format!("║{}║", r.pretty()))
                .collect::<Vec<String>>()
                .join("\n"),
        );
        pp.push_str("\n╚═══════════╝");
        pp
    }
}

impl Hnfen for Board {
    fn as_hnfen(&self) -> String {
        let mut buf = String::new();
        buf.push_str(
            &self
                .ranks
                .iter()
                .map(Rank::as_hnfen)
                .collect::<Vec<String>>()
                .join(RANK_SEP),
        );
        buf.push(' ');
        buf.push_str(&self.next.as_hnfen());
        buf
    }

    fn from_hnfen(hnfen: &str) -> Option<Self> {
        let splits: Vec<&str> = hnfen.split_whitespace().collect();
        Some(Board {
            ranks: splits[0]
                .split(RANK_SEP)
                .map(Rank::from_hnfen)
                .collect::<Option<Vec<Rank>>>()?
                .try_into()
                .unwrap(),
            next: if let Some(s) = splits.get(1) {
                Player::from_hnfen(s)?
            } else {
                Player::Black
            },
        })
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::from_hnfen(crate::DEFAULT_START_HNFEN).expect("default map should be good")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn default_board() {
        Board::default();
    }

    #[test]
    fn test_rank_to_hnfen() {
        let mut rank = Rank { fields: [None; 11] };
        assert_eq!(rank.as_hnfen(), "11");
        rank.fields[10] = Some(Piece::King);
        assert_eq!(rank.as_hnfen(), "10K");
        rank.fields[0] = Some(Piece::Normal(Player::Black));
        assert_eq!(rank.as_hnfen(), "a9K");
        rank.fields[5] = Some(Piece::Normal(Player::White)); // a....h....K
        assert_eq!(rank.as_hnfen(), "a4h4K");
    }

    #[test]
    fn test_rank_from_hnfen() {
        let mut expected_success_cases = vec!["11", "10K", "K10", "a9K", "a4h4K"];
        expected_success_cases.extend(
            crate::DEFAULT_START_HNFEN
                .split_ascii_whitespace()
                .next()
                .unwrap()
                .split(RANK_SEP),
        );
        for case in expected_success_cases.into_iter() {
            assert_eq!(Rank::from_hnfen(case).unwrap().as_hnfen(), case);
        }

        assert_eq!(Rank::from_hnfen("00011").unwrap().as_hnfen(), "11");
        assert_eq!(Rank::from_hnfen("0a09a0").unwrap().as_hnfen(), "a9a");
    }

    #[test]
    fn test_board_from_hnfen() {
        assert_eq!(
            Board::from_hnfen(crate::DEFAULT_START_HNFEN)
                .unwrap()
                .as_hnfen(),
            crate::DEFAULT_START_HNFEN
        );
    }

    #[test]
    fn get_pieces_amount() {
        let board = Board::default();
        assert_eq!(board.pieces(Player::White).len(), 13);
        assert_eq!(board.pieces(Player::Black).len(), 24);
    }
}
