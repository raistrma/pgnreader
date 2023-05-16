use std::fmt;

// No PGN standard defined for Delay/Bronstien
#[derive(Debug, Clone)]
pub enum TimeControlIncrement {
    Added { added_seconds_per_move: u32 }, // aka Increment aka Bonus aka Fischer Time. Time added to a players clock after thier move
    Delay { delay_seconds_per_move: u32 }, // aka simple delay aka US Delay. Clock waits delay after last move before restarting
    Bronstien { delay_seconds_per_move: u32 }, // effectively equivalent to delay, just a differnce in how clock shows the delay. Adds the increment after a move is played as the minimum of the time spent on the more or the increment, whichever is smaller i.e. clock will never increase after a move, at most the player will get back the time they spent.
}

impl fmt::Display for TimeControlIncrement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeControlIncrement::Added { added_seconds_per_move } => write!(f, "+{}", added_seconds_per_move),
            TimeControlIncrement::Delay { delay_seconds_per_move } => write!(f, "+{}{{delay}}", delay_seconds_per_move),
            TimeControlIncrement::Bronstien { delay_seconds_per_move } => write!(f, "+{}{{Bronstien type delay}}", delay_seconds_per_move),
        }
    }
}

// No PGN standard defined for Correspondance
// Must be parsed in the given order to avoid conflict
#[derive(Debug, Clone)]
pub enum TimeControlPeriod {
    Unknown, // Time control is not known
    NoTimeControl, // Unlimited time for the game
    Correspondance { // Move must be completed within given time, no limit on overall length of game, i.e. daily chess
        move_time_seconds: u32,
    },
    HourGlass { // Time decreases whilst making a move with a commensurate increase in opponents time, a move must be made before time runs out
        move_time_seconds: u32,
    },
    Incremental { // Game must be completed within period plus increment per move
        period_length_seconds: u32,
        increment: TimeControlIncrement,
    },
    MovesPerPeriod { // X number of moves must be made before the end of the period with optional increment per move
        moves: u8,
        period_length_seconds: u32,
        increment: Option<TimeControlIncrement>,
        next_period: Option<Box<TimeControlPeriod>>,
    },    
    SuddenDeath { // Game must be completed within period
        period_length_seconds: u32,
    },
}

// Output a time control based on the PGN standard, section 9.6.1

impl fmt::Display for TimeControlPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeControlPeriod::NoTimeControl => write!(f, "-"),
            TimeControlPeriod::Correspondance {
                move_time_seconds,
            } => write!(f, "?{{{} seconds per move}}", move_time_seconds),
            TimeControlPeriod::Unknown => write!(f, "?"),
            TimeControlPeriod::HourGlass {
                move_time_seconds,
            } => write!(f, "*{}", move_time_seconds),
            TimeControlPeriod::Incremental {
                period_length_seconds,
                increment,
            } => write!(f, "{}{}", period_length_seconds, increment),
            TimeControlPeriod::MovesPerPeriod {
                moves,
                period_length_seconds,
                increment,
                next_period,
            } => {
                write!(f, "{}/{}", moves, period_length_seconds)?;
                if let Some(increment) = increment {
                    write!(f, "{}", increment)?;
                } 
                if let Some(period) = next_period {
                    write!(f, ":{}", period)
                } else {
                    write!(f, "")
                }
            },
            TimeControlPeriod::SuddenDeath {
                period_length_seconds,
            } => write!(f, "{}", period_length_seconds),
        }   
    }
}

#[cfg(test)]
mod tests {
// Mostly Tested in conjuction with the pgn_import module

    use super::*;

    #[test]
    fn time_control_test() {
        let tc2 = TimeControlPeriod::SuddenDeath { period_length_seconds: 900 };
        let tc = TimeControlPeriod::MovesPerPeriod { moves: 40, period_length_seconds: 7000, increment: Some(TimeControlIncrement::Bronstien { delay_seconds_per_move: 10 }), next_period: Some(Box::new(tc2))};
        let tc3 = TimeControlPeriod::MovesPerPeriod { moves: 20, period_length_seconds: 1000, increment: None, next_period: Some(Box::new(tc))};
        println!("{}", tc3);
    }

}

