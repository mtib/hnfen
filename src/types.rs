use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct Board {
    ranks: [Rank; 11],
    next: Player,
}

#[derive(Debug, Clone)]
pub struct Rank {
    fields: [Option<Piece>; 11],
}

const WHITE: &str = "h";
const BLACK: &str = "a";
const KING: &str = "K";
const RANK_SEP: &str = "/";

#[derive(Debug, Clone, Copy)]
pub enum Piece {
    Normal(Player),
    King,
}

#[derive(Debug, Clone, Copy)]
pub enum Player {
    /// Starts, is attacker
    Black,
    /// Second, is defender
    White,
}

pub trait Hnfen {
    fn as_hnfen(&self) -> String;
    fn from_hnfen(hnfen: &str) -> Self;
}

impl Hnfen for Player {
    fn as_hnfen(&self) -> String {
        match self {
            Player::Black => BLACK,
            Player::White => WHITE,
        }
        .to_string()
    }

    fn from_hnfen(hnfen: &str) -> Self {
        match hnfen {
            BLACK => Player::Black,
            WHITE => Player::White,
            _ => panic!("Unexpected input character: {}", hnfen),
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

    fn from_hnfen(hnfen: &str) -> Self {
        match hnfen {
            BLACK => Piece::Normal(Player::Black),
            WHITE => Piece::Normal(Player::White),
            KING => Piece::King,
            _ => panic!("Unexpected input character: {}", hnfen),
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

    fn from_hnfen(hnfen: &str) -> Self {
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
                _ => groups.push(C::Character(Piece::from_hnfen(&k.to_string()))),
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
            panic!("Rank not exactly filled by hnfen: {}", hnfen);
        }

        rank
    }
}

impl Default for Rank {
    fn default() -> Self {
        Rank { fields: [None; 11] }
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

    fn from_hnfen(hnfen: &str) -> Self {
        let splits: Vec<&str> = hnfen.split_whitespace().collect();
        Board {
            ranks: splits[0]
                .split(RANK_SEP)
                .map(Rank::from_hnfen)
                .collect::<Vec<Rank>>()
                .try_into()
                .unwrap(),
            next: if let Some(s) = splits.get(1) {
                Player::from_hnfen(s)
            } else {
                Player::Black
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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
            assert_eq!(Rank::from_hnfen(case).as_hnfen(), case);
        }

        assert_eq!(Rank::from_hnfen("00011").as_hnfen(), "11");
        assert_eq!(Rank::from_hnfen("0a09a0").as_hnfen(), "a9a");
    }

    #[test]
    fn test_board_from_hnfen() {
        assert_eq!(
            Board::from_hnfen(crate::DEFAULT_START_HNFEN).as_hnfen(),
            crate::DEFAULT_START_HNFEN
        );
    }
}
