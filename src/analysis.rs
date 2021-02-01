use log::{error, warn};

use thiserror::Error;

/// InfoParseError captures possible info parsing errors
#[derive(Error, Debug)]
pub enum InfoParseError {
    #[error("could not parse number for key '{0}' from info")]
    ParseNumberError(String),
    #[error("invalid info key '{0}'")]
    InvalidKeyError(String),
    #[error("invalid score specifier '{0}'")]
    InvalidScoreSpecifier(String),
}

/// log info parse error and return it as a result
pub fn info_parse_error(err: InfoParseError) -> Result<(), InfoParseError> {
    error!("{:?}", err);

    Err(err)
}

/// log parse number error and return it as a result
pub fn parse_number_error<T: AsRef<str>>(key: T) -> Result<(), InfoParseError> {
    let key = key.as_ref().to_string();

    info_parse_error(InfoParseError::ParseNumberError(key))
}

/// generate string buffer with given name and size
macro_rules! gen_str_buff {
	($(#[$attr:meta] => $type:ident, $size:expr),*) => { $(
	    #[$attr]
	    #[derive(Clone, Copy)]
		pub struct $type {
			pub len: usize,
			pub buff: [u8; $size],
		}

		#[$attr]
		#[doc = "implementation"]
		impl $type {
			#[doc = "create new"]
			#[$attr]
			pub fn new() -> Self {
				Self {
					len: 0,
					buff: [0; $size]
				}
			}

			#[doc = "convert"]
			#[$attr]
			#[doc = "to option ( None if empty, Some(contents) otherwise )"]
			pub fn to_opt(self) -> Option<String> {
				if self.len == 0 {
					return None;
				}

				Some(String::from(self))
			}

			#[doc = "set"]
			#[$attr]
			#[doc = "( value will be trimmed to buffer size )"]
			pub fn set<T: AsRef<str>>(&mut self, value: T) -> Self {
				let bytes = value.as_ref().as_bytes();

				let mut len = bytes.len();

				if len > $size{
					len = $size;
				}

				self.len = len;

				self.buff[0..len].copy_from_slice(&bytes[0..len]);

				*self
			}

			#[doc = "reset"]
			#[$attr]
			#[doc = "to empty buffer"]
			pub fn reset(&mut self) -> Self {
				self.len = 0;

				*self
			}

			pub fn set_trim<T: AsRef<str>>(&mut self, value: T, trim: char) -> Self {
				let value_ref = value.as_ref();
				let value_string = value_ref.to_string();
				let bytes = value_ref.as_bytes();

				let mut total_len = value_string.len();

			    value_ref.to_string().chars().rev().take_while(|c| {
			        total_len -= 1;
			        ( *c != trim ) || ( total_len > $size )
			    }).collect::<String>().len();

			    self.len = total_len;

			    self.buff[0..total_len].copy_from_slice(&bytes[0..total_len]);

				*self
			}
		}

		#[doc = "implement From<&str> for"]
		#[$attr]
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

		#[doc = "implement From<String> for"]
		#[$attr]
		impl std::convert::From<String> for $type {
			fn from(value: String) -> Self {
				Self::from(value.as_str())
			}
		}

		#[doc = "implement From<"]
		#[$attr]
		#[doc = "> for String"]
		impl std::convert::From<$type> for String {
			fn from(buff: $type) -> String {
				std::str::from_utf8(&buff.buff[0..buff.len]).unwrap().to_string()
			}
		}

		#[doc = "implement Display for"]
		#[$attr]
		impl std::fmt::Display for $type {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		        write!(f, "{}", String::from(*self))
		    }
		}

		#[doc = "implement Debug for"]
		#[$attr]
		impl std::fmt::Debug for $type {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		        write!(f, "[{}[{}]: '{}']", stringify!($type), self.len, String::from(*self))
		    }
		}
	)* }
}

/// maximum length of uci move
const UCI_MAX_LENGTH: usize = 5;
/// typical length of uci move
const UCI_TYPICAL_LENGTH: usize = 4;
/// maximum number of pv moves to store
#[cfg(not(test))]
const MAX_PV_MOVES: usize = 10;
#[cfg(test)]
const MAX_PV_MOVES: usize = 2;
/// pv buffer size
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
    /// centipawn
    Cp(i32),
    /// mate
    Mate(i32),
}

/// analysis info
#[derive(Debug, Clone, Copy)]
pub struct AnalysisInfo {
    /// false for ongoing analysis, true when analysis stopped on bestmove received
    pub done: bool,
    /// best move
    bestmove: UciBuff,
    /// ponder
    ponder: UciBuff,
    /// pv
    pv: PvBuff,
    /// multipv
    pub multipv: usize,
    /// depth
    pub depth: usize,
    /// seldepth
    pub seldepth: usize,
    /// tbhits
    pub tbhits: u64,
    /// nodes
    pub nodes: u64,
    /// time
    pub time: usize,
    /// nodes per second
    pub nps: u64,
    /// score ( centipawns or mate )
    pub score: Score,
}

/// parsing state
#[derive(Debug)]
#[allow(dead_code)]
// TODO: make this pub(crate)
pub enum ParsingState {
    Info,
    Key,
    Unknown,
    Multipv,
    Depth,
    Seldepth,
    Tbhits,
    Nodes,
    Time,
    Nps,
    Score,
    ScoreCp,
    ScoreMate,
    PvBestmove,
    PvPonder,
    PvRest,
}

/// analysis info implementation
impl AnalysisInfo {
    /// create new analysis info
    pub fn new() -> Self {
        Self {
            done: false,
            bestmove: UciBuff::new(),
            ponder: UciBuff::new(),
            pv: PvBuff::new(),
            multipv: 0,
            depth: 0,
            seldepth: 0,
            tbhits: 0,
            nodes: 0,
            time: 0,
            nps: 0,
            score: Score::Cp(0),
        }
    }

    // get bestmove
    pub fn bestmove(self) -> Option<String> {
        self.bestmove.to_opt()
    }

    // get ponder
    pub fn ponder(self) -> Option<String> {
        self.ponder.to_opt()
    }

    // get pv
    pub fn pv(self) -> Option<String> {
        self.pv.to_opt()
    }

    /// parse info string
    pub fn parse<T: std::convert::AsRef<str>>(&mut self, info: T) -> Result<(), InfoParseError> {
        let info = info.as_ref();
        let mut ps = ParsingState::Info;
        let mut pv_buff = String::new();
        let mut pv_on = false;
        let mut first_string = true;

        for token in info.split(" ") {
            match ps {
                ParsingState::Info => {
                    match token {
                        "info" => ps = ParsingState::Key,
                        _ => {
                            // not an info
                            return Ok(());
                        }
                    }
                }
                ParsingState::Key => {
                    if token == "string" {
                        // anything starting with 'info string' is not analysis info rather verbal information to user
                        if first_string {
                            return Ok(());
                        } else {
                            // occuring later in key position 'string' is not a valid analysis info token
                            return Err(InfoParseError::InvalidKeyError(token.to_string()));
                        }
                    }

                    ps = match token {
                        "multipv" => ParsingState::Multipv,
                        "depth" => ParsingState::Depth,
                        "seldepth" => ParsingState::Seldepth,
                        "tbhits" => ParsingState::Tbhits,
                        "nodes" => ParsingState::Nodes,
                        "time" => ParsingState::Time,
                        "nps" => ParsingState::Nps,
                        "score" => ParsingState::Score,
                        "pv" => ParsingState::PvBestmove,
                        // don't hang parsing at unknown token for the moment
                        // TODO: consider making this an error
                        _ => ParsingState::Unknown,
                    }
                }
                ParsingState::Score => match token {
                    "cp" => ps = ParsingState::ScoreCp,
                    "mate" => ps = ParsingState::ScoreMate,
                    _ => {
                        // not a valid score specifier
                        return info_parse_error(InfoParseError::InvalidScoreSpecifier(
                            token.to_string(),
                        ));
                    }
                },
                ParsingState::Unknown => {
                    // ignore this token and hope for the best ( namely that it had a single token arg )
                    warn!("unknown info key {}", token);

                    ps = ParsingState::Key
                }
                _ => {
                    match ps {
                        ParsingState::Multipv => match token.parse::<usize>() {
                            Ok(multipv) => self.multipv = multipv,
                            _ => return parse_number_error(token),
                        },
                        ParsingState::Depth => match token.parse::<usize>() {
                            Ok(depth) => self.depth = depth,
                            _ => return parse_number_error(token),
                        },
                        ParsingState::Seldepth => match token.parse::<usize>() {
                            Ok(seldepth) => self.seldepth = seldepth,
                            _ => return parse_number_error(token),
                        },
                        ParsingState::Tbhits => match token.parse::<u64>() {
                            Ok(tbhits) => self.tbhits = tbhits,
                            _ => return parse_number_error(token),
                        },
                        ParsingState::Nodes => match token.parse::<u64>() {
                            Ok(nodes) => self.nodes = nodes,
                            _ => return parse_number_error(token),
                        },
                        ParsingState::Nps => match token.parse::<u64>() {
                            Ok(nps) => self.nps = nps,
                            _ => return parse_number_error(token),
                        },
                        ParsingState::Time => match token.parse::<usize>() {
                            Ok(time) => self.time = time,
                            _ => return parse_number_error(token),
                        },
                        ParsingState::ScoreCp => match token.parse::<i32>() {
                            Ok(score_cp) => self.score = Score::Cp(score_cp),
                            _ => return parse_number_error(token),
                        },
                        ParsingState::ScoreMate => match token.parse::<i32>() {
                            Ok(score_mate) => self.score = Score::Mate(score_mate),
                            _ => return parse_number_error(token),
                        },
                        ParsingState::PvBestmove => {
                            pv_buff = pv_buff + token;

                            self.bestmove = UciBuff::from(token);

                            self.ponder.reset();

                            pv_on = true;

                            ps = ParsingState::PvPonder
                        }
                        ParsingState::PvPonder => {
                            pv_buff = pv_buff + " " + token;

                            self.ponder = UciBuff::from(token);

                            ps = ParsingState::PvRest
                        }
                        ParsingState::PvRest => pv_buff = pv_buff + " " + token,
                        _ => {
                            // should not happen
                        }
                    }

                    // anything from key pv onwards should be added to pv
                    // otherwise switch back to parsing key
                    if !pv_on {
                        ps = ParsingState::Key;
                    }
                }
            }

            first_string = false;
        }

        self.pv.set_trim(pv_buff, ' ');

        Ok(())
    }
}

#[test]
fn set_trim() {
    let mut x = PvBuff::new().set("e2e4");

    assert_eq!(x.len, 4);

    assert_eq!(String::from(x), "e2e4".to_string());

    x.set_trim("e2e4 e7e5 g1f3 b8c6", ' ');

    assert_eq!(x.len, 9);

    assert_eq!(String::from(x), "e2e4 e7e5".to_string());
}

#[test]
fn parse_error() {
    let mut ai = AnalysisInfo::new();

    let _ = ai.parse(
        "info depth 3 score mate 5 nodes 3000000000 time 3000 nps 1000000 pv e2e4 e7e5 g1f3",
    );

    assert_eq!(ai.depth, 3);
    assert_eq!(format!("{:?}", ai.score), format!("{:?}", Score::Mate(5)));
    assert_eq!(format!("{:?}", ai.ponder()), format!("{:?}", Some("e7e5")));
}
