pub mod pgn_import;
pub mod pgn_export;
use crate::definitions::*;
use crate::time_controls::*;

use std::fmt;

#[derive(Debug)]
pub enum CheckType {
    Check,
    CheckMate,
    StaleMate,
}

#[derive(Debug, Clone, Copy)]
pub enum Castle {
    KingSide,
    QueenSide,
}

#[derive(Debug, Clone)]
// SAN = Standard Algebraic Notation for a Move
pub struct SANPlyCoordinates {
    from_file: Option<File>,
    from_rank: Option<Rank>,
    to_square: Square,
}

#[derive(Debug, Clone)]
pub enum SANply {
    Basic {
        piece_moved: PieceType,
        mv: SANPlyCoordinates,
    },
    Capture {
        piece_moved: PieceType,
        mv: SANPlyCoordinates,
    },
    Promotion {
        mv: SANPlyCoordinates,
        piece_promoted: PieceType,
    },
    CapturePromotion {
        mv: SANPlyCoordinates,
        piece_promoted: PieceType,
    },
    Castle(Castle),
}

#[derive(Debug)]
pub struct PGNmove {
    white_ply: SANply,
    white_ply_annotation: Option<String>,
    black_ply: Option<SANply>,
    black_ply_annotation: Option<String>,
}

// Contains a ordered vector of moves
#[derive(Debug)]
pub struct PGNmovetext {
    moves: Vec<PGNmove>,
}

#[derive(Debug)]
pub struct PGNDateTag {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
}

#[derive(Debug)]
pub struct PGNTimeTag {
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
}

#[derive(Debug)]
pub enum PGNRoundTag {
    Unknown,
    NotApplicable,
    Name(String)
}

#[derive(Debug, Clone)]
pub enum PGNGameTerminationMarker {
    WhiteWins,
    BlackWins,
    Draw,
    Undetermined,
}

#[derive(Debug)]
pub struct PGNGenericTagPair {
    tag: String,
    value: String,
}

#[derive(Debug)]
pub struct PGNTagPairRoster {
    event: Option<String>,
    site: Option<String>,
    date: PGNDateTag,
    round: PGNRoundTag,
    white: Option<String>,
    black: Option<String>,
    result: PGNGameTerminationMarker,
    time: PGNTimeTag,
    time_control: TimeControlPeriod,
    fen_string: Option<String>,
    other_tag_pairs: Vec<PGNGenericTagPair>,
}

impl Default for PGNTagPairRoster {
    fn default() -> Self {
        PGNTagPairRoster {
            event: None,
            site: None,
            date: PGNDateTag{ year: None, month: None, day: None },
            round: PGNRoundTag::NotApplicable,
            white: None,
            black: None,
            result: PGNGameTerminationMarker::Undetermined,
            time: PGNTimeTag{ hour: None, minute: None, second: None },
            time_control: TimeControlPeriod::Unknown,
            fen_string: None,
            other_tag_pairs: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct PGNFile {
    tag_pair_roster: PGNTagPairRoster,
    movetext: PGNmovetext,
    game_termination_marker: PGNGameTerminationMarker,
}

#[cfg(test)]
mod tests {

    // use super::*;

}