use super::*;

// Movetext Section
impl fmt::Display for SANPlyCoordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(file) = &self.from_file { write!(f, "{}", file)?; }
        if let Some(rank) = &self.from_rank { write!(f, "{}", rank)?; }
        write!(f, "{}", self.to_square)
    }
}

impl fmt::Display for SANply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SANply::Basic { piece_moved, mv }
                if (*piece_moved == PieceType::PawnsWhite) || (*piece_moved == PieceType::PawnsBlack) => { write!(f, "{}", mv) }
            SANply::Basic { piece_moved, mv } => write!(f, "{}{}", piece_moved, mv),
            SANply::Capture { piece_moved, mv,
            } if (*piece_moved == PieceType::PawnsWhite) || (*piece_moved == PieceType::PawnsBlack) =>
            {
                match &mv.from_file {
                    Some(file) => write!(f, "{}x{}", file, mv.to_square),
                    None => panic!("An error occurred parsing SANply::Capture: file must be provided for pawn captures"),
                }
            }
            SANply::Capture { piece_moved, mv,
            } => {
                let from_file = if let Some(file) = mv.from_file { FILE_LABELS[file as usize].to_string() } else { "".to_string() };
                let from_rank = if let Some(rank) = mv.from_rank { RANK_LABELS[rank as usize].to_string() } else { "".to_string() };
                write!(f, "{}{}{}x{}", piece_moved, from_file, from_rank, mv.to_square)
            },
            SANply::Promotion { mv, piece_promoted: piece_promotion,
            } => write!(f, "{}={}", mv, piece_promotion),
            SANply::CapturePromotion { mv, piece_promoted: piece_promotion,
            } => {
                match &mv.from_file {
                    Some(file) => write!(f, "{}x{}={}", file, mv.to_square, piece_promotion),
                    None => panic!("An error occurred parsing SANply::CapturePromotion: file must be provided for pawn captures"),
                }
            }
            SANply::Castle(castle_direction) => match castle_direction {
                Castle::KingSide => write!(f, "O-O"),
                Castle::QueenSide => write!(f, "O-O-O"),
            },
        }
    }
}

impl fmt::Display for PGNmove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.white_ply)?;
        if let Some(white_ply_annotation) = &self.white_ply_annotation { write!(f, "{}", white_ply_annotation)?; }
        if let Some(black_move) = &self.black_ply { write!(f, " {}", black_move)?; }
        if let Some(black_ply_annotation) = &self.black_ply_annotation { write!(f, "{}", black_ply_annotation)?; }
        write!(f, "")
    }
}

impl fmt::Display for PGNmovetext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut move_count: usize = 1usize;
        for mv in &self.moves {
            write!(f, "{}. {} ", move_count, mv)?;
            move_count += 1;
        }
        write!(f, "")
    }
}

//Tag Pair Section

impl fmt::Display for PGNDateTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.year {
            Some(year) => write!(f, "{:04}", year)?,
            None => write!(f, "????")?,
        };
        match self.month {
            Some(month) => write!(f, ".{:02}", month)?,
            None  => write!(f, ".??")?,
        };
        match self.day {
            Some(day) => write!(f, ".{:02}", day),
            None  => write!(f, ".??"),
        }
    }
}

impl fmt::Display for PGNTimeTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.hour {
            Some(hour) => write!(f, "{:02}", hour)?,
            None => write!(f, "??")?,
        };
        match self.minute {
            Some(minute) => write!(f, ":{:02}", minute)?,
            None  => write!(f, ":??")?,
        };
        match self.second {
            Some(second) => write!(f, ":{:02}", second),
            None  => write!(f, ":??"),
        }
    }
}

impl fmt::Display for PGNRoundTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PGNRoundTag::Unknown => write!(f, "?"),
            PGNRoundTag::NotApplicable => write!(f, "-"),
            PGNRoundTag::Name(round) => write!(f, "{}", round),
        }
    }
}

impl fmt::Display for PGNGameTerminationMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PGNGameTerminationMarker::WhiteWins => write!(f, "1-0"),
            PGNGameTerminationMarker::BlackWins => write!(f, "0-1"),
            PGNGameTerminationMarker::Draw => write!(f, "1/2-1/2"),
            PGNGameTerminationMarker::Undetermined => write!(f, "*"),
        }
    }
}

impl fmt::Display for PGNGenericTagPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} \"{}\"]", self.tag, self.value)
    }
}

impl fmt::Display for PGNTagPairRoster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.event {
            Some(event) => writeln!(f, "[Event \"{}\"]", event)?,
            None => writeln!(f, "[Event \"?\"]")?,
        };
        match &self.site {
            Some(site) => writeln!(f, "[Site \"{}\"]", site)?,
            None => writeln!(f, "[Site \"?\"]")?,
        };
        writeln!(f, "[Date \"{}\"]", self.date)?;
        writeln!(f, "[Round \"{}\"]", self.round)?;
        match &self.white {
            Some(white) => writeln!(f, "[White \"{}\"]", white)?,
            None => writeln!(f, "[White \"?\"]")?,
        };
        match &self.black {
            Some(black) => writeln!(f, "[Black \"{}\"]", black)?,
            None => writeln!(f, "[Black \"?\"]")?,
        };
        writeln!(f, "[Result \"{}\"]", self.result)?;
        writeln!(f, "[Time \"{}\"]", self.time)?;
        writeln!(f, "[TimeControl \"{}\"]", self.time_control)?;
        match &self.fen_string {
            Some(fen_string) => {
                writeln!(f, "[Setup \"1\"]")?;
                writeln!(f, "[FEN \"{}\"]", fen_string)?;
            },
            None => writeln!(f, "[Setup \"0\"]")?,
        };
        for tag_pair in &self.other_tag_pairs {
            writeln!(f, "{}", tag_pair)?;
        };
        write!(f, "")
    }
}

//Overall PGN File
impl fmt::Display for PGNFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag_pair_roster)?;
        write!(f, "\n{}", self.movetext)?;
        write!(f, "{}", self.game_termination_marker)
    }
}

#[cfg(test)]
mod tests {
// Mostly Tested in conjuction with the pgn_import module

use super::*;

#[should_panic]
    #[test]
    fn import_sanply_failure_test1() {

        let move_input = SANply::Capture { 
            piece_moved: PieceType::PawnsWhite, 
            mv: SANPlyCoordinates { from_file: None, from_rank: None, to_square: Square { reference: 34 } } 
        };

        let _ = move_input.to_string();

    }

    #[should_panic]
    #[test]
    fn import_sanply_failure_test2() {

        let move_input = SANply::CapturePromotion { 
            piece_promoted: PieceType::Queen, 
            mv: SANPlyCoordinates { from_file: None, from_rank: None, to_square: Square { reference: 34 } } 
        };

        let _ = move_input.to_string();
        
    }

}