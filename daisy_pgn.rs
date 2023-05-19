#![allow(unused_imports)]

use std::{hint::black_box, str::FromStr};

use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, error::*, multi::*,
    sequence::*, *,
};

use daisychain::prelude::ParseError as DParseError;
use daisychain::prelude::*;

use crate::pgn::*; 
use crate::definitions::*;

pub fn parse_name<'a, E: nom::error::ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    terminated(take_until("@"), tag("@"))(input)
}

pub fn parse_domain_part<'a, E: nom::error::ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    terminated(take_until("."), tag("."))(input)
}

pub fn parse_domain_end<'a, E: nom::error::ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    recognize(tuple((alpha1, alphanumeric0)))(input)
}

pub fn parse_domain<'a, E: nom::error::ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    recognize(tuple((many1(parse_domain_part), parse_domain_end)))(input)
}

pub fn parse_email<'a, E: nom::error::ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    recognize(tuple((parse_name, parse_domain)))(input)
}

pub fn daisychain_parse_email(
    input: &str,
) -> Result<(&str, String), daisychain::prelude::ParseError> {
    let (cursor, name, domain_part, domain_end) = Cursor::from(input)
        .chars_not_in(1.., &['@'])
        .parse_selection::<String>()
        .text("@")
        .alphabetics(1..)
        .parse_selection::<String>()
        .text(".")
        .alphabetics(1..)
        .parse_selection::<String>()
        .validate()?;

    let email = name + &domain_part + &domain_end;

    Ok((cursor.str()?, email))
}

impl FromStr for File {
    type Err = DParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            "c" => Ok(Self::C),
            "d" => Ok(Self::D),
            "e" => Ok(Self::E),
            "f" => Ok(Self::F),
            "g" => Ok(Self::G),
            "h" => Ok(Self::H),
            _ => Err(DParseError::NoMatch {
                action: "matching file",
                args: "",
            }),
        }
    }
}

pub fn parse_file(s: &str) -> Result<(&str, Option<File>), DParseError> {
    if let Ok((c, file)) = Cursor::from(s)
        .chars_in(0..=1, &FILE_LABELS)
        .parse_selection::<File>()
        .validate() {
            Ok((c.str()?, Some(file)))
        } else {
            Ok((s, None))
        }
}

impl FromStr for Rank {
    type Err = DParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a" => Ok(Self::R1),
            "b" => Ok(Self::R2),
            "c" => Ok(Self::R3),
            "d" => Ok(Self::R4),
            "e" => Ok(Self::R5),
            "f" => Ok(Self::R6),
            "g" => Ok(Self::R7),
            "h" => Ok(Self::R8),
            _ => Err(DParseError::NoMatch {
                action: "matching rank",
                args: "",
            }),
        }
    }
}

pub fn parse_rank(s: &str) -> Result<(&str, Option<Rank>), DParseError> {
    if let Ok((c, rank)) = Cursor::from(s)
        .chars_in(0..=1, &RANK_LABELS)
        .parse_selection::<Rank>()
        .validate() {
            Ok((c.str()?, Some(rank)))
        } else {
            Ok((s, None))
        }
}

impl FromStr for PieceType {
    type Err = DParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "K" => Ok(Self::King),
            "Q" => Ok(Self::Queen),
            "R" => Ok(Self::Rook),
            "B" => Ok(Self::Bishop),
            "N" => Ok(Self::Knight),
            "wP" => Ok(Self::PawnsWhite),
            "bP" => Ok(Self::PawnsBlack),
            _ => Err(DParseError::NoMatch {
                action: "matching piece",
                args: "",
            }),
        }
    }
}

impl FromStr for SANply {
    type Err = DParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        let c = Cursor::from(s);

        let (c1, piece) = if let Ok((c1, piece)) = c
            .clone()
            .text_alt(&MAJOR_PIECETYPES)
            .parse_selection::<PieceType>()
            .validate() {
                (c1, Some(piece))
            } else {
                (c, None)
            };
        
        let (c1, piece) = if let Ok((c1, piece)) = c
            .clone()
            .text_alt(&MAJOR_PIECETYPES)
            .parse_selection::<PieceType>()
            .validate() {
                (c1, Some(piece))
            } else {
                (c, None)
            };

        Ok(SANply::Castle(Castle::KingSide))
    }
}

pub fn parse_opt_comment(s: &str) -> Result<(&str, ()), DParseError> {
    Ok((s, ()))
}

pub fn parse_opt_comment_with_move_num(s: &str) -> Result<(&str, ()), DParseError> {
    Ok((s, ()))
}

pub fn parse_pgn_move(s: &str) -> Result<(&str, PGNmove), DParseError> {
    let (c, _move_num, _comment, white_ply) = Cursor::from(s)
        .digits(1..)
        .parse_selection::<u8>()
        .text(".")
        .debug_context("white move")
        .ws()
        .parse_with_str(parse_opt_comment)
        .ws()
        .non_ws()
        .parse_selection::<SANply>()
        .validate()?;

    let (c, _comment, black_ply) = c
        .debug_context("black move")
        .ws()
        .parse_with_str(parse_opt_comment_with_move_num)
        .ws()
        .non_ws()
        .parse_selection::<SANply>()
        .select(|c| c.digits(1..).text(".").digits(2..=2))
        .validate()?;

    Ok((
        c.str()?,
        PGNmove {
            white_ply,
            white_ply_annotation: None,
            black_ply: Some(black_ply),
            black_ply_annotation: None,
        },
    ))
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use super::*;
    use log::{debug, trace};
    use nom::error;
    use test_log::test;

    #[test]
    fn daisy_pgn_test() {
        let san_move = "1. O-O O-O";

        let (c, pgn_move) = parse_pgn_move(san_move).unwrap();

        assert_eq!(c, "");
        assert_eq!(pgn_move.white_ply, SANply::Castle(Castle::KingSide));
        assert_eq!(pgn_move.black_ply, Some(SANply::Castle(Castle::KingSide)));

    }

    #[test]
    fn test_parse_email() {
        let email = "mark.raistrick@gmail.com\n";

        let tricky_email = "mark.raistrick@gmail.1com.com.123\n";

        // let (input, output) = parse_email::<nom::error::Error<_>>(email).unwrap();

        // debug!("input remaining: {input}, output found: {output}");

        // let (input, output) = parse_email::<nom::error::Error<_>>(tricky_email).unwrap();

        // debug!("input remaining: {input}, output found: {output}");

        let (input, output) = daisychain_parse_email(email).unwrap();

        debug!("input remaining: {input}, output found: {output}");
    }

    #[derive(PartialEq, Debug)]
    pub struct Time {
        pub hours: u32,
        pub mins: u32,
    }

    impl Time {
        pub fn new(hours: u32, mins: u32) -> Self {
            Self { hours, mins }
        }
    }

    impl FromStr for Time {
        type Err = daisychain::prelude::ParseError;

        /// eg "09:23" or "23:59"
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (_cur, hours, mins) = Cursor::from(s)
                .digits(2..=2)
                .parse_selection::<u32>() // daisychain will use u32::FromStr
                .text(":")
                .digits(2..=2)
                .parse_selection() // often no need to specify type explicitly
                .end_of_stream() // ensure we are at end-of-string
                .validate()?;
            Ok(Time { hours, mins })
        }
    }

    #[test]
    fn test_parse_0923() {
        trace!(target:"mark", "HELLO {}", 5);
        assert_eq!(Time::from_str("09:23").unwrap(), Time::new(9, 23));
        assert!(Time::from_str("09+23").is_err());
        assert!(Time::from_str("09:23X").is_err());
    }
}
