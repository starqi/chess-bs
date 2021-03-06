use super::move_list::*;
use super::entities::*;
use super::coords::*;

/// Invariant: Move snapshot order is by column
/// Size 2 arrays are indexed by `Player` enum numbers
pub struct CastleUtils {
    pub oo_move_snapshots: [MoveSnapshot; 2],
    pub ooo_move_snapshots: [MoveSnapshot; 2],
    pub oo_king_traversal_sqs: [[Coord; 2]; 2],
    pub ooo_king_traversal_sqs: [[Coord; 3]; 2]
}

impl CastleUtils {

    fn get_oo_move_snapshot_for_row(player: Player) -> MoveSnapshot {
        let row = player.get_first_row();
        return MoveSnapshot([
            Some((Coord(4, row), BeforeAfterSquares(Square::Occupied(Piece::King, player), Square::Blank))),
            Some((Coord(5, row), BeforeAfterSquares(Square::Blank, Square::Occupied(Piece::Rook, player)))),
            Some((Coord(6, row), BeforeAfterSquares(Square::Blank, Square::Occupied(Piece::King, player)))),
            Some((Coord(7, row), BeforeAfterSquares(Square::Occupied(Piece::Rook, player), Square::Blank))),
            None
        ], 0., MoveDescription::Oo);
    }

    fn get_ooo_move_snapshot_for_row(player: Player) -> MoveSnapshot {
        let row = player.get_first_row();
        return MoveSnapshot([
            Some((Coord(0, row), BeforeAfterSquares(Square::Occupied(Piece::Rook, player), Square::Blank))),
            Some((Coord(1, row), BeforeAfterSquares(Square::Blank, Square::Blank))),
            Some((Coord(2, row), BeforeAfterSquares(Square::Blank, Square::Occupied(Piece::King, player)))),
            Some((Coord(3, row), BeforeAfterSquares(Square::Blank, Square::Occupied(Piece::Rook, player)))),
            Some((Coord(4, row), BeforeAfterSquares(Square::Occupied(Piece::King, player), Square::Blank)))
        ], 0., MoveDescription::Ooo);
    }

    pub fn new() -> CastleUtils {
        crate::console_log!("Generating castle constants");

        let white_first_row = Player::get_first_row(Player::White);
        let black_first_row = Player::get_first_row(Player::Black);

        return CastleUtils {
            oo_move_snapshots: [CastleUtils::get_oo_move_snapshot_for_row(Player::White), CastleUtils::get_oo_move_snapshot_for_row(Player::Black)],
            ooo_move_snapshots: [CastleUtils::get_ooo_move_snapshot_for_row(Player::White), CastleUtils::get_ooo_move_snapshot_for_row(Player::Black)],
            oo_king_traversal_sqs: [
                [Coord(6, white_first_row), Coord(5, white_first_row)],
                [Coord(6, black_first_row), Coord(5, black_first_row)]
            ],
            ooo_king_traversal_sqs: [
                [Coord(1, white_first_row), Coord(2, white_first_row), Coord(3, white_first_row)],
                [Coord(1, black_first_row), Coord(2, black_first_row), Coord(3, black_first_row)]
            ]
        };
    }
}
