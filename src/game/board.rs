
use std::collections::HashSet;
use std::fmt::{Display, Formatter, self};
use super::coords::*;
use super::entities::*;
use super::move_list::*;
use super::castle_utils::*;
use super::basic_move_test::*;

#[derive(Clone)]
pub struct PlayerState {
    // TODO First thing to switch into bitboards
    pub piece_locs: HashSet<Coord>,
    pub can_oo: bool,
    pub can_ooo: bool
}

impl PlayerState {
    fn new() -> PlayerState {
        PlayerState {
            piece_locs: HashSet::new(),
            can_oo: true,
            can_ooo: true
        }
    }
}

#[derive(Clone)]
pub struct Board {
    player_with_turn: Player,
    d: [Square; 64],
    player_state: [PlayerState; 2]
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        for i in 0..self.d.len() {
            if i % 8 == 0 && i != 0 {
                write!(f, "\n")?;
            }
            write!(f, "{}", self.d[i])?;
        }
        Ok(())
    }
}

impl Board {
    pub fn new() -> Board {
        let mut board = Board {
            d: [Square::Blank; 64],
            player_with_turn: Player::White,
            player_state: [PlayerState::new(), PlayerState::new()]
        };
        board.set_standard_rows();
        board
    }

    //////////////////////////////////////////////////
    // Player state

    pub fn get_player_with_turn(&self) -> Player {
        self.player_with_turn
    }

    pub fn get_player_state(&self, player: Player) -> &PlayerState {
        &self.player_state[player as usize]
    }

    fn get_player_state_mut(&mut self, player: Player) -> &mut PlayerState {
        &mut self.player_state[player as usize]
    }

    //////////////////////////////////////////////////
    // Get set squares

    pub fn get_safe(&self, file: char, rank: u8) -> Result<Square, Error> {
        let Coord(x, y) = file_rank_to_xy_safe(file, rank)?;
        Ok(self.get_by_xy(x, y))
    }

    pub fn get_by_xy_safe(&self, x: i32, y: i32) -> Result<Square, Error> {
        check_i32_xy(x, y)?;
        Ok(self.get_by_xy(x as u8, y as u8))
    }

    pub fn get_by_xy(&self, x: u8, y: u8) -> Square {
        return self.d[y as usize * 8 + x as usize];
    }

    pub fn set_by_xy(&mut self, x: u8, y: u8, s: Square) {
        if let Square::Occupied(_, occupied_player) = self.get_by_xy(x, y) {
            let piece_list = &mut self.get_player_state_mut(occupied_player).piece_locs;
            piece_list.remove(&Coord(x, y));
        }

        if let Square::Occupied(_, new_player) = s {
            let piece_list = &mut self.get_player_state_mut(new_player).piece_locs;
            piece_list.insert(Coord(x, y));
        }

        self.d[y as usize * 8 + x as usize] = s;
    }

    pub fn set(&mut self, file: char, rank: u8, s: Square) {
        let Coord(x, y) = file_rank_to_xy(file, rank);
        self.set_by_xy(x, y, s);
    }

    //////////////////////////////////////////////////
    // Moves

    pub fn undo_move(&mut self, m: &MoveSnapshot) {
        for sq in m.iter() {
            if let Some((Coord(x, y), before_after)) = sq {
                self.set_by_xy(*x, *y, before_after.0);
            }
        }
        self.player_with_turn = self.player_with_turn.get_other_player();
    }

    pub fn make_move(&mut self, m: &MoveSnapshot) {
        for sq in m.iter() {
            if let Some((Coord(x, y), before_after)) = sq {
                self.set_by_xy(*x, *y, before_after.1);
            }
        }
        self.player_with_turn = self.player_with_turn.get_other_player();
    }

    /// Gets the final set of legal moves
    pub fn get_moves(&mut self, castle_utils: &CastleUtils, temp_moves: &mut MoveList, result: &mut MoveList) {
        temp_moves.write_index = 0;
        BasicMoveTest::fill_player(self.player_with_turn, self, false, temp_moves);
        BasicMoveTest::filter_check_threats(
            self,
            self.player_with_turn.get_other_player(), 
            temp_moves,
            0,
            temp_moves.write_index,
            result
        );

        // TODO Orig position check
        /*
        let (can_oo, can_ooo) = {
            let ps = self.get_player_state(self.player_with_turn);
            (ps.can_oo, ps.can_ooo)
        };

        if can_oo {
            self.push_castle(
                &castle_utils.oo_king_traversal_sqs[self.player_with_turn as usize],
                &castle_utils.oo_move_snapshots[self.player_with_turn as usize],
                self.player_with_turn,
                temp_moves,
                result
            );
        }
        if can_ooo {
            self.push_castle(
                &castle_utils.ooo_king_traversal_sqs[self.player_with_turn as usize],
                &castle_utils.ooo_move_snapshots[self.player_with_turn as usize],
                self.player_with_turn,
                temp_moves,
                result
            );
        }
        */
    }

    /// Assumes castle has not already been done
    /// Separate from the normal candidate move + check threat pattern
    fn push_castle(
        &mut self,
        king_travel_squares: &[Coord],
        move_snapshot: &MoveSnapshot,
        player_with_turn: Player,
        temp_moves: &mut MoveList,
        result: &mut MoveList
    ) {
        for Coord(x, y) in king_travel_squares.iter() {
            if let Square::Occupied(_, _) = self.get_by_xy(*x, *y) {
                return;
            } 
        }

        for Coord(x, y) in king_travel_squares.iter() {
            self.set_by_xy(*x, *y, Square::Occupied(Piece::King, player_with_turn));
        }
        temp_moves.write_index = 0;
        BasicMoveTest::fill_player(
            player_with_turn.get_other_player(), self, true, temp_moves
        );
        let can_castle = !BasicMoveTest::has_king_capture_move(temp_moves, 0, temp_moves.write_index, player_with_turn);
        for Coord(x, y) in king_travel_squares.iter() {
            self.set_by_xy(*x, *y, Square::Blank);
        }

        if can_castle {
            result.copy_and_write(move_snapshot);
        }
    }

    //////////////////////////////////////////////////


    fn set_uniform_row(&mut self, rank: u8, player: Player, piece: Piece) {
        for i in 0..8 {
            self.set_by_xy(i, 8 - rank, Square::Occupied(piece, player));
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

    fn set_standard_rows(&mut self) {
        self.set_main_row(1, Player::White);
        self.set_uniform_row(2, Player::White, Piece::Pawn);
        self.set_main_row(8, Player::Black);
        self.set_uniform_row(7, Player::Black, Piece::Pawn);
    }
}