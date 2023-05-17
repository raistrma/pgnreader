use crate::time_controls::*;

use super::*;

use nom::{
    *,
    error::*,
    combinator::*,
    sequence::*,
    bytes::complete::*,
    character::complete::*, 
    multi::*, 
    branch::*,
  };

// Parse Comments and unwanted data, functions throw away the PGN data
pub fn parse_comment_rest_of_line<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E>{
    value((), tuple((tag(";"), not_line_ending, line_ending, multispace0)))(input)
}

pub fn parse_comment_braced<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    value((), tuple((tag("{"), take_until("}"), tag("}"), multispace0)))(input)
}

pub fn parse_escape_mechanism<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    value((), tuple((opt(line_ending), tag("%"), not_line_ending, line_ending, multispace0)))(input)
}

// Will fail for recursive varations as it stands 
pub fn parse_annotation_variation<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    value((), tuple((tag("("), take_until(")"), tag(")"), multispace0)))(input)                    
}

pub fn parse_commentry<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    value((), many1(alt((
        parse_comment_rest_of_line, 
        parse_comment_braced, 
        parse_escape_mechanism, 
        parse_annotation_variation,
    ))))(input)
}

// Parse Movetext
pub fn parse_move_number<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, u32, E> {
    terminated(preceded(multispace0, u32), pair(char('.'), multispace0))(input)
}

pub fn parse_move_number_after_annotation<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, u32, E> {
    terminated(preceded(multispace0, u32), pair(tag("..."), multispace0))(input)
}

// Parse SAN
pub fn parse_piece<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, PieceType, E> {
    alt((
        value(PieceType::King, char('K')),
        value(PieceType::Queen, char('Q')),
        value(PieceType::Rook, char('R')),
        value(PieceType::Bishop, char('B')),
        value(PieceType::Knight, char('N')),
    ))(input)
}

pub fn parse_file<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, File, E> {
    alt((
        value(File::A, char('a')),
        value(File::B, char('b')),
        value(File::C, char('c')),
        value(File::D, char('d')),
        value(File::E, char('e')),
        value(File::F, char('f')),
        value(File::G, char('g')),
        value(File::H, char('h')),
    ))(input)
}

pub fn parse_rank<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Rank, E> {
    alt((
        value(Rank::R1, char('1')),
        value(Rank::R2, char('2')),
        value(Rank::R3, char('3')),
        value(Rank::R4, char('4')),
        value(Rank::R5, char('5')),
        value(Rank::R6, char('6')),
        value(Rank::R7, char('7')),
        value(Rank::R8, char('8')),
    ))(input)
}

pub fn parse_square<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Square, E> {
    map(pair(parse_file, parse_rank), | (file, rank) | Square { reference: ((rank as u8) * 8 + (file as u8)) as usize})(input)
}

pub fn parse_capture<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, bool, E> {
    value(true, char('x'))(input)
}

pub fn parse_promotion<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, PieceType, E> {
    alt((
        value(PieceType::Queen, tag("=Q")),
        value(PieceType::Rook, tag("=R")),
        value(PieceType::Bishop, tag("=B")),
        value(PieceType::Knight, tag("=N")),
    ))(input)
}

pub fn parse_checks_and_nag<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, String, E> {
    map(recognize(many0(none_of( "= {(;\t\n\r"))), | check_nag_and_end_token: &str | check_nag_and_end_token.to_string())(input)
}

fn parse_san_castle<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SANply, E> {
    alt((
        value(SANply::Castle(Castle::QueenSide), tag("O-O-O")),
        value(SANply::Castle(Castle::KingSide), tag("O-O"))
    ))(input)
}

fn parse_san_capture_promotion<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SANply, E> {
    let (input, (from_file, to_square, piece_promoted)) = tuple((
        parse_file,
        preceded(parse_capture, parse_square),
        parse_promotion,
    ))(input)?;
    let mv: SANPlyCoordinates = SANPlyCoordinates {from_file: Some(from_file), from_rank: None, to_square};
    Ok((input, SANply::CapturePromotion { mv, piece_promoted }))
}

fn parse_san_promotion<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SANply, E> {
    let (input, (to_square, piece_promoted)) = tuple((
        parse_square,
        parse_promotion,
    ))(input)?;
    let mv: SANPlyCoordinates = SANPlyCoordinates {from_file: None, from_rank: None, to_square};
    Ok((input, SANply::Promotion { mv, piece_promoted }))
}

fn parse_san_capture<'a, E: ParseError<&'a str>>(input: &'a str, default_piece: PieceType) -> IResult<&'a str, SANply, E> {
    
    let (input, (piece, from_file, from_rank, to_square)) = tuple((
        opt(parse_piece),
        opt(parse_file),
        opt(parse_rank),
        preceded(parse_capture, parse_square),
    ))(input)?;
    
    let mv: SANPlyCoordinates = SANPlyCoordinates {from_file, from_rank, to_square };
    let piece_moved = piece.unwrap_or(default_piece);
    Ok((input, SANply::Capture { piece_moved, mv }))

}

fn parse_san_capture_white<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SANply, E> {
    parse_san_capture(input, PieceType::PawnsWhite)
}
fn parse_san_capture_black<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SANply, E> {
    parse_san_capture(input, PieceType::PawnsBlack)
}

fn parse_san_basic_qualified<'a, E: ParseError<&'a str>>(input: &'a str, default_piece: PieceType) -> IResult<&'a str, SANply, E> {
    
    let (input, (piece, from_file, from_rank, to_square)) = tuple((
        opt(parse_piece),
        opt(parse_file),
        opt(parse_rank),
        parse_square,
    ))(input)?;
    
    let mv: SANPlyCoordinates = SANPlyCoordinates { from_file, from_rank, to_square };
    let piece_moved = piece.unwrap_or(default_piece);
    Ok((input, SANply::Basic { piece_moved, mv }))

}

fn parse_san_basic_qualified_white<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SANply, E> {
    parse_san_basic_qualified(input, PieceType::PawnsWhite)
}
fn parse_san_basic_qualified_black<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SANply, E> {
    parse_san_basic_qualified(input, PieceType::PawnsBlack)
}

fn parse_san_basic_unqualified<'a, E: ParseError<&'a str>>(input: &'a str, default_piece: PieceType) -> IResult<&'a str, SANply, E> {
    
    let (input, (piece, to_square)) = tuple((
        opt(parse_piece),
        parse_square,
    ))(input)?;
    
    let mv: SANPlyCoordinates = SANPlyCoordinates {from_file: None, from_rank: None, to_square };
    let piece_moved = piece.unwrap_or(default_piece);
    Ok((input, SANply::Basic { piece_moved, mv }))

}

fn parse_san_basic_unqualified_white<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SANply, E> {
    parse_san_basic_unqualified(input, PieceType::PawnsWhite)
}
fn parse_san_basic_unqualified_black<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, SANply, E> {
    parse_san_basic_unqualified(input, PieceType::PawnsBlack)
}

pub fn parse_san_ply_white<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (SANply, Option<String>), E> {
    tuple((
        alt((
            parse_san_castle, 
            parse_san_capture_promotion, 
            parse_san_promotion, 
            parse_san_capture_white,
            parse_san_basic_qualified_white,
            parse_san_basic_unqualified_white,
        )),
        terminated(
            opt(parse_checks_and_nag),
            multispace0)
    ))(input)
}

pub fn parse_san_ply_black<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (SANply, Option<String>), E> {
    tuple((
        alt((
            parse_san_castle, 
            parse_san_capture_promotion, 
            parse_san_promotion, 
            parse_san_capture_black,
            parse_san_basic_qualified_black,
            parse_san_basic_unqualified_black,
        )),
        terminated(
            opt(parse_checks_and_nag),
            multispace0)
    ))(input)
}

pub fn parse_san_move<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, PGNmove, E> {

    let (input, (
        (white_ply, white_ply_annotation), 
        _comment1,
        black_ply_and_annotation,
        _comment2,
    )) = tuple((
        preceded(parse_move_number, parse_san_ply_white), 
        opt(pair(parse_commentry, parse_move_number_after_annotation)),
        opt(parse_san_ply_black),
        opt(parse_commentry),
    ))(input)?;
    match black_ply_and_annotation {
        Some((black_ply, black_ply_annotation)) => Ok((input, PGNmove { white_ply, white_ply_annotation, black_ply: Some(black_ply), black_ply_annotation })),
        None => Ok((input, PGNmove { white_ply, white_ply_annotation, black_ply: None, black_ply_annotation: None })),
    }

}

pub fn parse_san_game_termination_marker<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, PGNGameTerminationMarker, E> {
    alt((
        value(PGNGameTerminationMarker::WhiteWins, tag("1-0")),
        value(PGNGameTerminationMarker::BlackWins, tag("0-1")),
        value(PGNGameTerminationMarker::Draw, tag("1/2-1/2")),
        value(PGNGameTerminationMarker::Undetermined, tag("*")),
    ))(input)
}

pub fn parse_san_movetext<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, PGNmovetext, E> {
    map(many1(parse_san_move), | val | PGNmovetext {moves: val})(input)
    
}

// Parse Tag Pair Date/Times

pub fn parse_tag_pair_datetime_part<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Option<u8>, E> {
    alt((
        value(None, tag("??")),
        map(u8, Some)
    ))(input)
}

pub fn parse_tag_pair_year_part<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Option<u16>, E> {
    alt((
        value(None, tag("????")),
        map(u16, Some)
    ))(input)
}

pub fn parse_tag_pair_time<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, PGNTimeTag, E> {
    let (input, (hour, minute, second)) = tuple((
        terminated(parse_tag_pair_datetime_part,tag(":")),
        terminated(parse_tag_pair_datetime_part,tag(":")),
        parse_tag_pair_datetime_part,
    ))(input)?;
    Ok((input, PGNTimeTag { hour, minute, second }))
}

pub fn parse_tag_pair_date<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, PGNDateTag, E> {
    let (input, (year, month, day)) = tuple((
        terminated(parse_tag_pair_year_part,tag(".")),
        terminated(parse_tag_pair_datetime_part, tag(".")),
        parse_tag_pair_datetime_part,
    ))(input)?;
    Ok((input, PGNDateTag { year, month, day }))
}

// Parse Time Control Tag Pair

pub fn parse_tag_pair_timecontrol_increment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TimeControlIncrement, E> {
    alt((
        map(
            delimited(tag("+"), u32, tag("{delay}")), 
            | delay_seconds_per_move | TimeControlIncrement::Delay {delay_seconds_per_move }),
        map(
            delimited(tag("+"), u32, tag("{Bronstien type delay}")), 
            | delay_seconds_per_move | TimeControlIncrement::Bronstien {delay_seconds_per_move }),
        map(
            preceded(tag("+"), u32), 
            | added_seconds_per_move | TimeControlIncrement::Added {added_seconds_per_move })
    ))(input)
}

pub fn parse_tag_pair_timecontrol_no_time_control<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TimeControlPeriod, E> {
    value(TimeControlPeriod::NoTimeControl, tag("-"))(input)
}

pub fn parse_tag_pair_timecontrol_correspondance<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TimeControlPeriod, E> {
    map(
        delimited(tag("?{"),u32,tag(" seconds per move}")),
        | move_time_seconds | TimeControlPeriod::Correspondance { move_time_seconds }
    )(input)
}

pub fn parse_tag_pair_timecontrol_unknown<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TimeControlPeriod, E> {
    value(TimeControlPeriod::Unknown, tag("?"))(input)
}

pub fn parse_tag_pair_timecontrol_hourglass<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TimeControlPeriod, E> {
    map(
        preceded(tag("*"),u32),
        | move_time_seconds | TimeControlPeriod::HourGlass { move_time_seconds }
    )(input)
}

pub fn parse_tag_pair_timecontrol_incremental<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TimeControlPeriod, E> {
    map(
        pair(u32, parse_tag_pair_timecontrol_increment),
        | (period_length_seconds, increment) | TimeControlPeriod::Incremental { period_length_seconds, increment}
    )(input)
}

pub fn parse_tag_pair_timecontrol_moves_per_period<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TimeControlPeriod, E> {
    let (input, ((moves, period_length_seconds), increment, next_period)) = tuple((
        separated_pair(u32, tag("/"), u32),
        opt(parse_tag_pair_timecontrol_increment),
        opt(preceded(tag(":"),parse_tag_pair_timecontrol)),
    ))(input)?;
    let next_period = next_period.map(Box::new);
    Ok((input, TimeControlPeriod::MovesPerPeriod { moves: moves as u8, period_length_seconds, increment, next_period }))
}

pub fn parse_tag_pair_timecontrol_suddendeath<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TimeControlPeriod, E> {
    map(u32, | period_length_seconds | TimeControlPeriod::SuddenDeath { period_length_seconds })(input)
}

pub fn parse_tag_pair_timecontrol<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TimeControlPeriod, E> {
    alt((
        parse_tag_pair_timecontrol_no_time_control, 
        parse_tag_pair_timecontrol_correspondance, 
        parse_tag_pair_timecontrol_unknown, 
        parse_tag_pair_timecontrol_hourglass,
        parse_tag_pair_timecontrol_incremental,
        parse_tag_pair_timecontrol_moves_per_period,
        parse_tag_pair_timecontrol_suddendeath,
    ))(input)
}

// Parse Tag Pairs
pub fn parse_tag_value<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Option<&str>, E> {
    delimited(
        tag("\""),
        opt(escaped(is_not(r#"\""#), '\\', one_of(r#"""#))),
        tag("\""),
    )(input)
}

pub fn parse_tag_pair<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (&str, &str), E> {
    let (input, (tag, value)) = separated_pair(
        preceded(tag("["),is_not(" ")),
        multispace1,
        terminated(parse_tag_value,pair(tag("]"), opt(multispace0))),
    )(input)?;
    Ok((input, (tag, value.unwrap_or(""))))
}

pub fn parse_tag_pairs<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, PGNTagPairRoster, E> {

    let (input, tag_pairs) = many1(tuple((
        parse_tag_pair,
        opt(parse_commentry),
    )))(input)?;

    let mut tag_pair_roster = PGNTagPairRoster::default();

    for ((tag, value), _) in tag_pairs {
        match tag {
            "Event" => tag_pair_roster.event = Some(value.to_string()),
            "Site" => tag_pair_roster.site = Some(value.to_string()),
            "Date" => (_, tag_pair_roster.date) = parse_tag_pair_date::<E>(value).unwrap_or(("", PGNDateTag{ year: None, month: None, day: None })),
            "Round" => tag_pair_roster.round = match value {
                "?" => PGNRoundTag::Unknown,
                "-" => PGNRoundTag::NotApplicable,
                _ => PGNRoundTag::Name(value.to_string()),
            },
            "White" => tag_pair_roster.white = Some(value.to_string()),
            "Black" => tag_pair_roster.black = Some(value.to_string()),
            "Result" => (_, tag_pair_roster.result) = parse_san_game_termination_marker(value)?,
            "Time" => (_, tag_pair_roster.time) = parse_tag_pair_time::<E>(value).unwrap_or(("", PGNTimeTag{ hour: None, minute: None, second: None })),
            "TimeControl" => (_, tag_pair_roster.time_control) = parse_tag_pair_timecontrol(value)?,
            "Setup" | "SetUp" | "setup" => (),
            "FEN" => tag_pair_roster.fen_string = Some(value.to_string()),
            _ => tag_pair_roster.other_tag_pairs.push(PGNGenericTagPair{ tag: tag.to_string(), value: value.to_string() }),
        }
    }
    
    Ok((input, tag_pair_roster ))

}

// Parse whole PGN file
pub fn parse_pgn_file<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, PGNFile, E> {
    let (input, (tag_pair_roster, movetext, game_termination_marker)) = tuple((
        terminated(parse_tag_pairs, opt(parse_commentry)),
        parse_san_movetext,
        terminated(parse_san_game_termination_marker, opt(parse_commentry)),
    ))(input)?;
    Ok((input, PGNFile{ tag_pair_roster, movetext, game_termination_marker }))
}

#[cfg(test)]
mod tests {

    use super::*;

    fn import_movetext_test(movetext_input: &str, movetext_expected_output:&str) {

        let (remaining, output) = parse_san_movetext::<nom::error::Error<_>>(movetext_input).unwrap();

        assert_eq!(output.to_string(), movetext_expected_output);
        println!("Original Movetext:\n{}\nRemaining:\n\n{}\nProcessed:\n\n{}\n", movetext_input, remaining, output);
    }

    #[test]
    fn import_movetext_tests() {

        //Not a valid move sequence, testing san permutations
        let movetext_input = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6
        4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O-O 9. h3 Nb8 10. d4 Nbd7
        11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6 16. Bh4 c5 17. dxe5
        Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6
        23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5
        hxg5 29. b8=N Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f8=R Bc8 34. Kf2 Bf5
        35. Ra7 g8=B 36. Ra6+ Kc5 37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6
        Nf2 42. g4 Bd3 43. Re6 Qa6xb7# 44. fxg1=Q+ fxg8 45. Qd8xe7 Qa1h8 46. h8=Q h1=R 47. e1=B e8=N";
        let movetext_expected_output = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O-O 9. h3 Nb8 10. d4 Nbd7 11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6 16. Bh4 c5 17. dxe5 Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6 23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5 hxg5 29. b8=N Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f8=R Bc8 34. Kf2 Bf5 35. Ra7 g8=B 36. Ra6+ Kc5 37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6 Nf2 42. g4 Bd3 43. Re6 Qa6xb7# 44. fxg1=Q+ fxg8 45. Qd8xe7 Qa1h8 46. h8=Q h1=R 47. e1=B e8=N ";

        import_movetext_test(movetext_input, movetext_expected_output);

    }
    
    fn import_pgn_test(input: &str, expected_output: &str) {
        let (_, output) = parse_pgn_file::<nom::error::Error<_>>(input).unwrap();
        assert_eq!(output.to_string(), expected_output);
    }

    #[test]
    fn import_pgn_tests() {
        
        let input = "[Event \"F/S Return Match\"];
        [Site \"Belgrade, Serbia JUG\"]
        [Date \"1992.11.04\"]
        [Round \"29\"]
        [White \"Fischer, Robert J.\"]
        [Black \"Spassky, Boris V.\"]
        [Result \"1/2-1/2\"]
        [TimeControl \"?\"]

        1. e4!! e5!? 2. Nf3?! Nc6?? 3. Bb5$120 a6 {This opening is called the Ruy Lopez.}
        4. Ba4 Nf6 5. O-O Be7 6. Re1 b5{Another Random Comment.} 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7
        11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 (14. Bg5 b4 15. Nb1 h6) 14... b4 15. Nb1 h6 16. Bh4 c5 17. dxe5
        Nxe4 18. Bxe7 Qxe7 19. exd6 {Another Random Comment.} 19... Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6
        23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5
        hxg5 29. b3 Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8 34. Kf2 Bf5{Another Random Comment.}
        35. Ra7 g6 36. Ra6+ Kc5 37. Ke1{Another Random Comment.} 37... Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6
        Nf2 42. g4 Bd3 43. Re6# {Another Random Comment.}1/2-1/2";
        let expected_output = "[Event \"F/S Return Match\"]\n[Site \"Belgrade, Serbia JUG\"]\n[Date \"1992.11.04\"]\n[Round \"29\"]\n[White \"Fischer, Robert J.\"]\n[Black \"Spassky, Boris V.\"]\n[Result \"1/2-1/2\"]\n[Time \"??:??:??\"]\n[TimeControl \"?\"]\n[Setup \"0\"]\n\n1. e4!! e5!? 2. Nf3?! Nc6?? 3. Bb5$120 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7 11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6 16. Bh4 c5 17. dxe5 Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6 23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5 hxg5 29. b3 Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8 34. Kf2 Bf5 35. Ra7 g6 36. Ra6+ Kc5 37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6 Nf2 42. g4 Bd3 43. Re6# 1/2-1/2";
        import_pgn_test(input, expected_output);

        // Chess 960 Game PGN
        let input = "[Event \"Chess960: 2005 Fischer Random Dropout Tournament, Round 6\"] [Site \"SchemingMind.com\"] [Time \"14:06:56\"] [Date \"????.??.??\"] [Round \"-\"] [White \"gvhill\"] [Black \"saxon\"] [Result \"1-0\"] [Variant \"fischerandom\"] [SetUp \"1\"] [FEN \"rbbkqnnr/pppppppp/8/8/8/8/PPPPPPPP/RBBKQNNR w KQkq - 0 1\"] [WhiteCountry \"USA\"] [BlackCountry \"GER\"] [TimeControl \"123+456\"] 1. d4 { Congratulations on making the final round!  It's a pleasure to play you again. } 1... d5 2. Nf3 Nf6 3. Ne3 Ne6 4. c4 dxc4 5. Nxc4 c5 6. dxc5 Qd7+ 7. Qd2 Nxc5 8. Qxd7+ Bxd7 9. O-O O-O 10. Rd1 { I think we have a draw from here.  What do you think? } { looks like a very drawish position. After another 10 move we will end in an equal endgame. Not much excitment here.... } 1-0";
        let expected_output = "[Event \"Chess960: 2005 Fischer Random Dropout Tournament, Round 6\"]\n[Site \"SchemingMind.com\"]\n[Date \"????.??.??\"]\n[Round \"-\"]\n[White \"gvhill\"]\n[Black \"saxon\"]\n[Result \"1-0\"]\n[Time \"14:06:56\"]\n[TimeControl \"123+456\"]\n[Setup \"1\"]\n[FEN \"rbbkqnnr/pppppppp/8/8/8/8/PPPPPPPP/RBBKQNNR w KQkq - 0 1\"]\n[Variant \"fischerandom\"]\n[WhiteCountry \"USA\"]\n[BlackCountry \"GER\"]\n\n1. d4 d5 2. Nf3 Nf6 3. Ne3 Ne6 4. c4 dxc4 5. Nxc4 c5 6. dxc5 Qd7+ 7. Qd2 Nxc5 8. Qxd7+ Bxd7 9. O-O O-O 10. Rd1 1-0";
        import_pgn_test(input, expected_output);

        // Additional tests to mop up any remaining untested paths
        let input = "[Date \"1992.11.04\"]\n[Round \"?\"]\n[Result \"*\"]\n[Time \"??:??:??\"]\n[TimeControl \"23/45:10/10+12{delay}:255/456+123{Bronstien type delay}:*100\"]\n[Setup \"0\"]\n\n1. e4!! e5!? 2. Nf3?! Nc6?? 3. Bb5$120 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. 1c3 O-O 9. ah3 Nb8 10. d4 Nbd7 11. c4 hc6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 4h6 16. Bh4 c5 17. dxe5 Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6 23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5 hxg5 29. b3 Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8 34. Kf2 Bf5 35. Ra7 g6 36. Ra6+ Kc5 37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6 Nf2 42. g4 Bd3 43. Re6# 0-1";
        let expected_output = "[Event \"?\"]\n[Site \"?\"]\n[Date \"1992.11.04\"]\n[Round \"?\"]\n[White \"?\"]\n[Black \"?\"]\n[Result \"*\"]\n[Time \"??:??:??\"]\n[TimeControl \"23/45:10/10+12{delay}:255/456+123{Bronstien type delay}:*100\"]\n[Setup \"0\"]\n\n1. e4!! e5!? 2. Nf3?! Nc6?? 3. Bb5$120 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. 1c3 O-O 9. ah3 Nb8 10. d4 Nbd7 11. c4 hc6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 4h6 16. Bh4 c5 17. dxe5 Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6 23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5 hxg5 29. b3 Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8 34. Kf2 Bf5 35. Ra7 g6 36. Ra6+ Kc5 37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6 Nf2 42. g4 Bd3 43. Re6# 0-1";
        import_pgn_test(input, expected_output);

        let input = "[Event \"3rd Al Ain Chess Rapid\"]
        [Site \"Al-Ain UAE\"]
        [Date \"2014.12.18\"]
        [Round \"5.23\"]
        [White \"Ahadzada,Hajiali\"]
        [Black \"Firouzja,Alireza\"]
        [Result \"0-1\"]
        [TimeControl \"-\"]
        [WhiteElo \"\"]
        [BlackElo \"2332\"]
        [ECO \"C01\"]

        1.e4 e6 2.d4 d5 3.exd5 exd5 4.c4 Nf6 5.Nc3 Bb4 6.Bg5 O-O 7.Ne2 Re8 8.c5 h6
        9.Bh4 Nc6 10.a3 Bxc3+ 11.bxc3 g5 12.Bg3 Ne4 13.Qc1 Na5 14.Rb1 Bf5 15.Rb4 b6
        16.cxb6 axb6 17.f3 Nxg3 18.hxg3 c5 19.dxc5 bxc5 20.Rb5 Bd3 21.Rb2 Nc4 22.Ra2 Ra6
        23.Kf2 Qf6 24.g4 Qe5 25.Ng3 Bxf1 26.Rxf1 Rae6 27.Nf5 Nd6 28.Nxd6 Qxd6 29.a4 d4
        30.cxd4 cxd4 31.Qd1 d3 32.Kg1 Re2 33.Rxe2 Rxe2 34.Re1 Qb6+ 35.Kh1 Qf2  0-1";
        let expected_output = "[Event \"3rd Al Ain Chess Rapid\"]\n[Site \"Al-Ain UAE\"]\n[Date \"2014.12.18\"]\n[Round \"5.23\"]\n[White \"Ahadzada,Hajiali\"]\n[Black \"Firouzja,Alireza\"]\n[Result \"0-1\"]\n[Time \"??:??:??\"]\n[TimeControl \"-\"]\n[Setup \"0\"]\n[WhiteElo \"\"]\n[BlackElo \"2332\"]\n[ECO \"C01\"]\n\n1. e4 e6 2. d4 d5 3. exd5 exd5 4. c4 Nf6 5. Nc3 Bb4 6. Bg5 O-O 7. Ne2 Re8 8. c5 h6 9. Bh4 Nc6 10. a3 Bxc3+ 11. bxc3 g5 12. Bg3 Ne4 13. Qc1 Na5 14. Rb1 Bf5 15. Rb4 b6 16. cxb6 axb6 17. f3 Nxg3 18. hxg3 c5 19. dxc5 bxc5 20. Rb5 Bd3 21. Rb2 Nc4 22. Ra2 Ra6 23. Kf2 Qf6 24. g4 Qe5 25. Ng3 Bxf1 26. Rxf1 Rae6 27. Nf5 Nd6 28. Nxd6 Qxd6 29. a4 d4 30. cxd4 cxd4 31. Qd1 d3 32. Kg1 Re2 33. Rxe2 Rxe2 34. Re1 Qb6+ 35. Kh1 Qf2 0-1";
        let (remaining, output) = parse_pgn_file::<nom::error::Error<_>>(input).unwrap();
        println!("Input:\n\n{}\nRemaining:\n\n{}\nProcessed:\n\n{}", input, remaining, output);
        assert_eq!(output.to_string(), expected_output);

        let input = "[Event \"Schachbundesliga 2011-12\"]
        [Site \"Essen GER\"]
        [Date \"2011.12.11\"]
        [Round \"7\"]
        [White \"Volokitin,And\"]
        [Black \"Rabiega,R\"]
        [Result \"1/2-1/2\"]
        [TimeControl \"98765\"]
        [WhiteElo \"2686\"]
        [BlackElo \"2501\"]
        [ECO \"C67\"]

        1.e4 e5 2.Nf3 Nc6 3.Bb5 Nf6 4.O-O Nxe4 5.d4 Nd6 6.Bxc6 dxc6 7.dxe5 Nf5 8.Qxd8+ Kxd8
        9.h3 Be7 10.Nc3 Ke8 11.Ne2 h5 12.Re1 Be6 13.Nf4 Rd8 14.Nxe6 fxe6 15.c3 Rd5
        16.g3 Kf7 17.Kg2 Kg6 18.Bf4 Rhd8 19.Re2 a5 20.a4 R8d7 21.Rd2 Rd8 22.Rad1 Rxd2
        23.Rxd2 Rd5 24.g4 hxg4 25.hxg4 Nh4+ 26.Nxh4+ Bxh4 27.c4 Rd8 28.Rxd8 Bxd8
        29.Bd2 b6 30.b4 axb4 31.Bxb4 c5 32.Be1 Bg5 33.a5 bxa5 34.Bxa5 c6 35.Kg3 Bc1
        36.f4 Bb2 37.Be1 Kf7 38.Bf2 g5 39.Bxc5 gxf4+ 40.Kxf4 Kg6 41.Be3 Bc3 42.Ke4 Kf7
        43.Bd4 Be1 44.Kd3 Kg6 45.Bc3 Bf2 46.Bd2 Bb6 47.Kc2 Bc7 48.Bf4 Ba5 49.Kd3 Bc7
        50.Ke4 Ba5 51.Be3 Bc3 52.Kf4 Bb4 53.Bb6 Bd2+ 54.Ke4 Bc3 55.Bd8 Bb2 56.Bf6 Bc3
        57.Kd3 Be1 58.Kd4 Bf2+ 59.Kc3 Be1+ 60.Kb3 Bd2 61.Bd8 Be1 62.Kc2 Bg3 63.Bf6 Be1
        64.Kd3 Bg3 65.Ke3 Be1 66.Bd8 Bc3 67.Ke4 Be1 68.Bb6 Kg5 69.Kf3 Bc3 70.Be3+ Kg6
        71.Ke4 Ba5 72.Bf4 Bc3 73.Bc1 Ba5 74.Ba3 Be1 75.Be7 Bf2 76.Bb4 Kg5 77.Kf3 Bd4
        78.Bd2+ Kg6 79.Ke4 Bb6 80.Kf4 Bd4 81.Ba5 Bc5 82.Kf3 Kg5 83.Bd2+ Kg6 84.Ke4 Bb6
        85.Kd3 Bc7 86.Bf4 Bb6 87.Kc2 Ba5 88.Kb3 Be1 89.Bc1 Ba5 90.Ka4  1/2-1/2";
        let expected_output = "[Event \"Schachbundesliga 2011-12\"]\n[Site \"Essen GER\"]\n[Date \"2011.12.11\"]\n[Round \"7\"]\n[White \"Volokitin,And\"]\n[Black \"Rabiega,R\"]\n[Result \"1/2-1/2\"]\n[Time \"??:??:??\"]\n[TimeControl \"98765\"]\n[Setup \"0\"]\n[WhiteElo \"2686\"]\n[BlackElo \"2501\"]\n[ECO \"C67\"]\n\n1. e4 e5 2. Nf3 Nc6 3. Bb5 Nf6 4. O-O Nxe4 5. d4 Nd6 6. Bxc6 dxc6 7. dxe5 Nf5 8. Qxd8+ Kxd8 9. h3 Be7 10. Nc3 Ke8 11. Ne2 h5 12. Re1 Be6 13. Nf4 Rd8 14. Nxe6 fxe6 15. c3 Rd5 16. g3 Kf7 17. Kg2 Kg6 18. Bf4 Rhd8 19. Re2 a5 20. a4 R8d7 21. Rd2 Rd8 22. Rad1 Rxd2 23. Rxd2 Rd5 24. g4 hxg4 25. hxg4 Nh4+ 26. Nxh4+ Bxh4 27. c4 Rd8 28. Rxd8 Bxd8 29. Bd2 b6 30. b4 axb4 31. Bxb4 c5 32. Be1 Bg5 33. a5 bxa5 34. Bxa5 c6 35. Kg3 Bc1 36. f4 Bb2 37. Be1 Kf7 38. Bf2 g5 39. Bxc5 gxf4+ 40. Kxf4 Kg6 41. Be3 Bc3 42. Ke4 Kf7 43. Bd4 Be1 44. Kd3 Kg6 45. Bc3 Bf2 46. Bd2 Bb6 47. Kc2 Bc7 48. Bf4 Ba5 49. Kd3 Bc7 50. Ke4 Ba5 51. Be3 Bc3 52. Kf4 Bb4 53. Bb6 Bd2+ 54. Ke4 Bc3 55. Bd8 Bb2 56. Bf6 Bc3 57. Kd3 Be1 58. Kd4 Bf2+ 59. Kc3 Be1+ 60. Kb3 Bd2 61. Bd8 Be1 62. Kc2 Bg3 63. Bf6 Be1 64. Kd3 Bg3 65. Ke3 Be1 66. Bd8 Bc3 67. Ke4 Be1 68. Bb6 Kg5 69. Kf3 Bc3 70. Be3+ Kg6 71. Ke4 Ba5 72. Bf4 Bc3 73. Bc1 Ba5 74. Ba3 Be1 75. Be7 Bf2 76. Bb4 Kg5 77. Kf3 Bd4 78. Bd2+ Kg6 79. Ke4 Bb6 80. Kf4 Bd4 81. Ba5 Bc5 82. Kf3 Kg5 83. Bd2+ Kg6 84. Ke4 Bb6 85. Kd3 Bc7 86. Bf4 Bb6 87. Kc2 Ba5 88. Kb3 Be1 89. Bc1 Ba5 90. Ka4 1/2-1/2";
        let (remaining, output) = parse_pgn_file::<nom::error::Error<_>>(input).unwrap();
        println!("Input:\n\n{}\nRemaining:\n\n{}\nProcessed:\n\n{}", input, remaining, output);
        assert_eq!(output.to_string(), expected_output);

        let input = "[Event \"State Ch.\"]
        [Site \"New York, USA\"]
        [Date \"1910.??.??\"]
        [Round \"?\"]
        [White \"Capablanca\"]
        [Black \"Jaffe\"]
        [Result \"1-0\"]
        [ECO \"D46\"]
        [Opening \"Queen's Gambit Dec.\"]
        [Annotator \"Reinfeld, Fred\"]
        [WhiteTitle \"GM\"]
        [WhiteCountry \"Cuba\"]
        [BlackCountry \"United States\"]
        [Time \"??:??:??\"]
        [TimeControl \"?{1234 seconds per move}\"]

        1. d4 d5 2. Nf3 Nf6 3. e3 c6 4. c4 e6 5. Nc3 Nbd7 6. Bd3 Bd6
        7. O-O O-O 8. e4 dxe4 9. Nxe4 Nxe4 10. Bxe4 Nf6 11. Bc2 h6
        12. b3 b6 13. Bb2 Bb7 14. Qd3 g6 15. Rae1 Nh5 16. Bc1 Kg7
        17. Rxe6 Nf6 18. Ne5 c5 19. Bxh6+ Kxh6 20. Nxf7+ 1-0";
        let expected_output = "[Event \"State Ch.\"]\n[Site \"New York, USA\"]\n[Date \"1910.??.??\"]\n[Round \"?\"]\n[White \"Capablanca\"]\n[Black \"Jaffe\"]\n[Result \"1-0\"]\n[Time \"??:??:??\"]\n[TimeControl \"?{1234 seconds per move}\"]\n[Setup \"0\"]\n[ECO \"D46\"]\n[Opening \"Queen's Gambit Dec.\"]\n[Annotator \"Reinfeld, Fred\"]\n[WhiteTitle \"GM\"]\n[WhiteCountry \"Cuba\"]\n[BlackCountry \"United States\"]\n\n1. d4 d5 2. Nf3 Nf6 3. e3 c6 4. c4 e6 5. Nc3 Nbd7 6. Bd3 Bd6 7. O-O O-O 8. e4 dxe4 9. Nxe4 Nxe4 10. Bxe4 Nf6 11. Bc2 h6 12. b3 b6 13. Bb2 Bb7 14. Qd3 g6 15. Rae1 Nh5 16. Bc1 Kg7 17. Rxe6 Nf6 18. Ne5 c5 19. Bxh6+ Kxh6 20. Nxf7+ 1-0";
        let (remaining, output) = parse_pgn_file::<nom::error::Error<_>>(input).unwrap();
        println!("Input:\n\n{}\nRemaining:\n\n{}\nProcessed:\n\n{}", input, remaining, output);
        assert_eq!(output.to_string(), expected_output);

        let input = "[Event \"State Ch.\"]
        [Site \"New York, USA\"]
        [Date \"aaaa.??.??\"]
        [Round \"?\"]
        [White \"Capablanca\"]
        [Black \"Jaffe\"]
        [Result \"1-0\"]
        [ECO \"D46\"]
        [Opening \"Queen's Gambit Dec.\"]
        [Annotator \"Reinfeld, Fred\"]
        [WhiteTitle \"GM\"]
        [WhiteCountry \"Cuba\"]
        [BlackCountry \"United States\"]
        [Time \"??:gh:??\"]
        [TimeControl \"40/40\"]
        
        1. d4 d5 2. Nf3 Nf6 3. e3 c6 4. c4 e6 5. Nc3 Nbd7 6. Bd3 Bd6
        7. O-O O-O 8. e4 dxe4 9. Nxe4 Nxe4 10. Bxe4 Nf6 11. Bc2 h6
        12. b3 b6 13. Bb2 Bb7 14. Qd3 g6 15. Rae1 Nh5 16. Bc1 Kg7
        17. Rxe6 Nf6 18. Ne5 c5 19. Bxh6+ Kxh6 20. Nxf7+ 1-0";
                let expected_output = "[Event \"State Ch.\"]\n[Site \"New York, USA\"]\n[Date \"????.??.??\"]\n[Round \"?\"]\n[White \"Capablanca\"]\n[Black \"Jaffe\"]\n[Result \"1-0\"]\n[Time \"??:??:??\"]\n[TimeControl \"40/40\"]\n[Setup \"0\"]\n[ECO \"D46\"]\n[Opening \"Queen's Gambit Dec.\"]\n[Annotator \"Reinfeld, Fred\"]\n[WhiteTitle \"GM\"]\n[WhiteCountry \"Cuba\"]\n[BlackCountry \"United States\"]\n\n1. d4 d5 2. Nf3 Nf6 3. e3 c6 4. c4 e6 5. Nc3 Nbd7 6. Bd3 Bd6 7. O-O O-O 8. e4 dxe4 9. Nxe4 Nxe4 10. Bxe4 Nf6 11. Bc2 h6 12. b3 b6 13. Bb2 Bb7 14. Qd3 g6 15. Rae1 Nh5 16. Bc1 Kg7 17. Rxe6 Nf6 18. Ne5 c5 19. Bxh6+ Kxh6 20. Nxf7+ 1-0";
                let (remaining, output) = parse_pgn_file::<nom::error::Error<_>>(input).unwrap();
                println!("Input:\n\n{}\nRemaining:\n\n{}\nProcessed:\n\n{}", input, remaining, output);
                assert_eq!(output.to_string(), expected_output);

    }

}