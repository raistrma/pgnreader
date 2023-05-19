use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(usize)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

pub const FILE_LABELS: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", FILE_LABELS[*self as usize])
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(usize)]
pub enum Rank {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

pub const RANK_LABELS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", RANK_LABELS[*self as usize])
    }
}

pub const BOARD_SIZE: usize = 64;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Square {
    pub reference: usize,
}

pub const SQUARE_NAMES_BY_REF: [&str; BOARD_SIZE] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"
];

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", SQUARE_NAMES_BY_REF[self.reference])
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(usize)] // Consider using u16 or u32
pub enum PieceType {
    King = 0,
    Queen = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    PawnsWhite = 5,
    PawnsBlack = 6,
    RookCQ = 7,
    RookCK = 8,
    KingCB = 9,
    KingCQ = 10,
    KingCK = 11,
}

pub const COUNT_PIECETYPES: usize = 12;
pub const PIECE_NAMES_SHORT: [&str; COUNT_PIECETYPES] = ["K", "Q", "R", "B", "N", "P", "P", "R", "R", "K", "K", "K"];
pub const MAJOR_PIECETYPES: [&str; 5] = ["K", "Q", "R", "B", "N"];

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", PIECE_NAMES_SHORT[*self as usize])
    }
}