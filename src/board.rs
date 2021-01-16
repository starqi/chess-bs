// TODO King, en passante, promotion

use std::collections::HashSet;
use std::fmt::{Display, Formatter, self};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Piece {
    Pawn, Rook, Knight, Bishop, Queen, King
}

impl Piece {
    fn custom_fmt(&self, f: &mut Formatter<'_>, is_lower: bool) -> Result<(), fmt::Error> {
        let s = match self {
            Piece::Pawn => "P",
            Piece::Rook => "R",
            Piece::Knight => "N",
            Piece::Bishop => "B",
            Piece::Queen => "Q",
            Piece::King => "K"
        };

        if is_lower {
            write!(f, "{}", s.chars().nth(0).unwrap().to_lowercase())
        } else {
            write!(f, "{}", s)
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Player { 
    Black, White
}

#[derive(Copy, Clone, Debug)]
pub enum Square {
    Occupied(Piece, Player), Blank
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Square::Blank => {
                write!(f, " ")
            },
            Square::Occupied(piece, player) => {
                piece.custom_fmt(f, *player == Player::Black)
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    Unknown
}

pub struct MoveList {
    v: Vec<(char, u8)>,
    source: Option<(char, u8)>,
    revision: u32
}

impl MoveList {
    pub fn new() -> MoveList {
        MoveList { 
            v: Vec::new(),
            source: None,
            revision: 0 
        }
    }

    pub fn get_moves(&self) -> &Vec<(char, u8)> {
        &self.v
    }
}

#[derive(Debug)]
pub struct Board {
    pub player_with_turn: Player,
    arr: [Square; 64],
    revision: u32,
    black_piece_list: HashSet<(u8, u8)>,
    white_piece_list: HashSet<(u8, u8)>
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        for i in 0..self.arr.len() {
            if i % 8 == 0 && i != 0 {
                if let Err(fmt::Error) = write!(f, "\n") {
                    return Err(fmt::Error);
                }
            }
            if let Err(fmt::Error) = write!(f, "{}", self.arr[i]) {
                return Err(fmt::Error);
            }
        }
        Ok(())
    }
}

impl Board {
    pub fn new() -> Board {
        let mut board = Board {
            arr: [Square::Blank; 64],
            player_with_turn: Player::White,
            revision: 0,
            black_piece_list: HashSet::new(),
            white_piece_list: HashSet::new()
        };

        board.set_main_row(1, Player::White);
        board.set_pawn_row(2, Player::White);

        board.set_main_row(8, Player::Black);
        board.set_pawn_row(7, Player::Black);

        board
    }

    pub fn get_piece_locations(&self, player: Player) -> &HashSet<(u8, u8)> {
        match player {
            Player::White => &self.white_piece_list,
            Player::Black => &self.black_piece_list
        }
    }

    fn _get_piece_locations(&mut self, player: Player) -> &mut HashSet<(u8, u8)> {
        match player {
            Player::White => &mut self.white_piece_list,
            Player::Black => &mut self.black_piece_list
        }
    }

    pub fn set_by_xy(&mut self, x: u8, y: u8, s: Square) -> Option<Error> {
        if let Square::Occupied(_, occupied_player) = self.get_by_xy(x, y) {
            let piece_list: &mut HashSet<(u8, u8)> = self._get_piece_locations(occupied_player);
            piece_list.remove(&(x as u8, y as u8));
        }

        if let Square::Occupied(_, new_player) = s {
            let piece_list: &mut HashSet<(u8, u8)> = self._get_piece_locations(new_player);
            piece_list.insert((x as u8, y as u8));
        }

        self.arr[y as usize * 8 + x as usize] = s;
        self.revision += 1;

        None
    }

    pub fn set(&mut self, file: char, rank: u8, s: Square) -> Option<Error> {
        match Board::file_rank_to_xy_safe(file, rank) {
            Ok((x, y)) => self.set_by_xy(x, y, s),
            Err(e) => Some(e)
        }
    }

    pub fn get(&self, file: char, rank: u8) -> Result<Square, Error> {
        match Board::file_rank_to_xy_safe(file, rank) {
            Ok((x, y)) => Ok(self.get_by_xy(x, y)),
            Err(e) => Err(e)
        }
    }

    pub fn make_move(&mut self, moves: &mut MoveList, index: usize) -> Option<Error> {
        // TODO Extract
        let (source_file, source_rank) = match moves.source {
            None => {
                eprintln!("Move list is no longer valid");
                return Some(Error::Unknown);
            },
            Some(x) => x 
        };

        let source_square = match self.get(source_file, source_rank) {
            Err(e) => {
                eprintln!("Unexpected source square fetch failed - {} {}", source_file, source_rank);
                return Some(e);
            },
            Ok(x) => x
        };

        let (target_file, target_rank) = match moves.v.get(index) {
            None => {
                eprintln!("Move list index out of bounds {} / {}", index, moves.v.len());
                return Some(Error::Unknown);
            },
            Some(x) => x
        };

        self.set(*target_file, *target_rank, source_square);
        self.set(source_file, source_rank, Square::Blank);
        // TODO Extract
        moves.source = None;

        self.player_with_turn = match self.player_with_turn {
            Player::Black => Player::White,
            Player::White => Player::Black,
        };
        None
    }

    pub fn get_legal_moves(&self, file: char, rank: u8, result: &mut MoveList) -> Option<Error> {
        result.v.clear();
        result.revision = self.revision;
        result.source = Some((file, rank));

        let (x_u8, y_u8) = match Board::file_rank_to_xy_safe(file, rank) {
            Ok(r) => r,
            Err(e) => {
                return Some(e);
            }
        };
        let (x, y) = (x_u8 as i8, y_u8 as i8);

        let (piece, player) = match self.get_by_xy(x_u8, y_u8) {
            Square::Blank => {
                return None;
            },
            Square::Occupied(piece, player) => (piece, player)
        };

        match piece {
            Piece::Pawn => {
                let (y_delta, jump_row) = match player {
                    Player::Black => (1, 1),
                    Player::White => (-1, 6)
                };

                self.push_candidate(x, y + y_delta, player, &mut result.v);
                if y == jump_row {
                    self.push_candidate(x, y + y_delta * 2, player, &mut result.v);
                }

                for x_delta in -1..=1 {
                    if x_delta == 0 { continue; }

                    let x_p_delta: i8 = x + x_delta;
                    let y_p_delta: i8 = y + y_delta;

                    if x_p_delta < 0 || x_p_delta > 7 { continue; }
                    if y_p_delta < 0 || y_p_delta > 7 { continue; }

                    if let Square::Occupied(_, angled_player) = self.get_by_xy(x_p_delta as u8, y_p_delta as u8) {
                        if angled_player != player {
                            self.push_candidate(x + x_delta, y + y_delta, player, &mut result.v);
                        }
                    }
                }
            },
            Piece::Rook => {
                self.push_rook_candidates(x, y, player, &mut result.v);
            },
            Piece::Knight => {

                self.push_candidate(x - 1, y + 2, player, &mut result.v);
                self.push_candidate(x - 1, y - 2, player, &mut result.v);

                self.push_candidate(x - 2, y + 1, player, &mut result.v);
                self.push_candidate(x - 2, y - 1, player, &mut result.v);

                self.push_candidate(x + 2, y + 1, player, &mut result.v);
                self.push_candidate(x + 2, y - 1, player, &mut result.v);

                self.push_candidate(x + 1, y + 2, player, &mut result.v);
                self.push_candidate(x + 1, y - 2, player, &mut result.v);
            },
            Piece::Bishop => {
                self.push_bishop_candidates(x, y, player, &mut result.v);
            },
            Piece::Queen => {
                self.push_rook_candidates(x, y, player, &mut result.v);
                self.push_bishop_candidates(x, y, player, &mut result.v);
            },
            Piece::King => {
                for i in -1..=1 {
                    for j in -1..=1 {
                        if i == 0 && j == 0 {
                            continue;
                        }
                        self.push_candidate(x + i, y + j, player, &mut result.v);
                    }
                }
            }
        }

        None
    }

    fn push_rook_candidates(&self, x: i8, y: i8, player: Player, result: &mut Vec<(char, u8)>) {
        for _i in 1..=x {
            let i = x - _i;
            if self.push_candidate(i, y, player, result) { break; }
        }
        for i in x + 1..=7 {
            if self.push_candidate(i, y, player, result) { break; }
        }
        for _i in 1..=y {
            let i = y - _i;
            if self.push_candidate(x, i, player, result) { break; }
        }
        for i in y + 1..=7 {
            if self.push_candidate(x, i, player, result) { break; }
        }
    }

    fn push_bishop_candidates(&self, x: i8, y: i8, player: Player, result: &mut Vec<(char, u8)>) {
        for i in 1..=x {
            if self.push_candidate(x - i, y - i, player, result) { break; }
        }
        for i in 1..=x {
            if self.push_candidate(x - i, y + i, player, result) { break; }
        }
        for i in 1..=8 - (x + 1) {
            if self.push_candidate(x + i, y - i, player, result) { break; }
        }
        for i in 1..=8 - (x + 1) {
            if self.push_candidate(x + i, y + i, player, result) { break; }
        }
    }

    fn set_pawn_row(&mut self, rank: u8, player: Player) {
        for i in 0..8 {
            self.set_by_xy(i, 8 - rank, Square::Occupied(Piece::Pawn, player));
        }
    }

    fn set_main_row(&mut self, rank: u8, player: Player) {
        self.set('a', rank, Square::Occupied(Piece::Rook, player));
        self.set('b', rank, Square::Occupied(Piece::Knight, player));
        self.set('c', rank, Square::Occupied(Piece::Bishop, player));
        self.set('d', rank, Square::Occupied(Piece::Queen, player));
        self.set('e', rank, Square::Occupied(Piece::King, player));
        self.set('f', rank, Square::Occupied(Piece::Bishop, player));
        self.set('g', rank, Square::Occupied(Piece::Knight, player));
        self.set('h', rank, Square::Occupied(Piece::Rook, player));

    }
    // Checks are for blind arithmetic
    // Returns whether to terminate the direction (if applicable)
    fn push_candidate(&self, x: i8, y: i8, player_owner: Player, result: &mut Vec<(char, u8)>) -> bool {
        if player_owner == self.player_with_turn && x >= 0 && x <= 7 && y >= 0 && y <= 7 {
            let (moveable, terminate) = match self.get_by_xy(x as u8, y as u8) {
                Square::Occupied(piece, player) => {
                    (player != player_owner && piece != Piece::King, true)
                },
                Square::Blank => {
                    (true, false)
                }
            };
            if moveable {
                result.push(Board::xy_to_file_rank(x as u8, y as u8));
            }
            return terminate;
        } else {
            return true;
        }
    }

    fn get_by_xy(&self, x: u8, y: u8) -> Square {
        return self.arr[y as usize * 8 + x as usize];
    }

    //////////////////////////////////////////////////

    // FIXME
    pub fn xy_to_file_rank(x: u8, y: u8) -> (char, u8) {
        (std::char::from_u32(x as u32 + ('a' as u32)).unwrap(), 8 - (y as u8))
    }

    fn file_rank_to_xy(file: char, rank: u8) -> (u8, u8) {
        let x = file as u32 - 'a' as u32;
        let y = 8 - rank;
        (x as u8, y)
    }

    // Checks are for public interface
    fn file_rank_to_xy_safe(file: char, rank: u8) -> Result<(u8, u8), Error> {
        if rank < 1 || rank > 8 {
            eprintln!("Rank out of bounds - {}", rank);
            return Err(Error::Unknown);
        }
        let file_u32 = file as u32;
        if file_u32 < 'a' as u32 || file_u32 > 'h' as u32 {
            eprintln!("File out of bounds - {}", file);
            return Err(Error::Unknown);
        }
        return Ok(Board::file_rank_to_xy(file, rank));
    }

    fn get_other_player(player: Player) -> Player {
        match player {
            Player::Black => Player::White,
            Player::White => Player::Black
        }
    }
}
