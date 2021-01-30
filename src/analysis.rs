macro_rules! gen_str_buff {
	($(#[$attr:meta] => $type:ident, $size:expr),*) => { $(
	    #[$attr]
	    #[derive(Clone, Copy)]
		pub struct $type {
			pub len: usize,
			pub buff: [u8; $size],
		}

		impl $type {
			fn new() -> Self {
				Self {
					len: 0,
					buff: [0; $size]
				}
			}
		}

		impl std::convert::From<&str> for $type {
			fn from(value: &str) -> Self {
				let bytes = value.as_bytes();
				let mut len = bytes.len();
				if len > $size{
					len = $size;
				}

				let mut buff = $type::new();

                buff.len = len;
				buff.buff[0..len].copy_from_slice(&bytes[0..len]);

				buff
			}
		}

		impl std::convert::From<$type> for String {
			fn from(buff: $type) -> String {
				std::str::from_utf8(&buff.buff[0..buff.len]).unwrap().to_string()
			}
		}

		impl std::fmt::Display for $type {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		        write!(f, "{}", String::from(*self))
		    }
		}

		impl std::fmt::Debug for $type {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		        write!(f, "[{}: '{}']", stringify!($type), String::from(*self))
		    }
		}
	)* }
}

const UCI_MAX_LENGTH: usize = 5;
const UCI_TYPICAL_LENGTH: usize = 4;
const MAX_PV_MOVES: usize = 10;
const PV_BUFF_SIZE: usize = MAX_PV_MOVES * (UCI_TYPICAL_LENGTH + 1);

gen_str_buff!(
/// UciBuff
=> UciBuff, UCI_MAX_LENGTH,
/// PvBuff
=> PvBuff, PV_BUFF_SIZE
);

/// score
#[derive(Debug, Clone, Copy)]
pub enum Score {
    Cp(i32),
    Mate(i32),
}

use Score::*;

/// analysis info
#[derive(Debug, Clone, Copy)]
pub struct AnalysisInfo {
    /// best move
    bestmove: UciBuff,
    /// ponder
    ponder: UciBuff,
    /// pv
    pv: PvBuff,
    /// depth
    pub depth: usize,
    /// nodes
    pub nodes: u64,
    /// time
    pub time: usize,
    /// nodes per second
    pub nps: usize,
    /// score ( centipawns or mate )
    pub score: Score,
}

/// analysis info implementation
impl AnalysisInfo {
    /// create new analysis info
    pub fn new() -> Self {
        Self {
            bestmove: UciBuff::new(),
            ponder: UciBuff::new(),
            pv: PvBuff::new(),
            depth: 0,
            nodes: 0,
            time: 0,
            nps: 0,
            score: Cp(0),
        }
    }
}

/// parsing state
#[derive(Debug)]
#[allow(dead_code)]
// TODO: make this pub(crate)
pub enum ParsingState {
    Init,
    Unknown,
    Depth,
    Nodes,
    Time,
    Nps,
    Score,
    ScoreCp,
    ScoreMate,
    Pv,
}
