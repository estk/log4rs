use crate::encode::pattern::parser::Parser;
///! A module for parsing log path pattern to Chunks
use crate::encode::pattern::parser::Piece;

/// base and count
#[derive(Debug, PartialEq)]
pub struct Count {
    /// start index from base
    pub base: u32,
    /// totoal log file count
    pub count: u32,
}

/// struct for parsing log file name
/// for example "/some_dir/foo.log{c(%Y%m%d)}{c(1)(0)}"
#[derive(Debug)]
pub enum Chunk {
    /// parsed text, like "/some_dir/foo.log"
    Text(String),
    /// parsed time, like "%Y%m%d"
    Time(String),
    /// unknown filed
    Error(String),
    /// parsed coung, like {c(1)(0)} will parsed as Count { base:0 count:1 }
    Count(Count),
}

impl<'a> From<Piece<'a>> for Chunk {
    fn from(piece: Piece<'a>) -> Chunk {
        match piece {
            Piece::Text(text) => Chunk::Text(text.to_owned()),
            Piece::Error(err) => Chunk::Error(err),
            Piece::Argument {
                formatter,
                parameters: _,
            } => match formatter.name {
                "d" | "date" => {
                    if formatter.args.len() > 1 {
                        return Chunk::Error("expected at most two arguments".to_owned());
                    }

                    let format = match formatter.args.get(0) {
                        Some(arg) => {
                            let mut format = String::new();
                            for piece in arg {
                                match *piece {
                                    Piece::Text(text) => format.push_str(text),
                                    Piece::Argument { .. } => {
                                        format.push_str("{ERROR: unexpected formatter}");
                                    }
                                    Piece::Error(ref err) => {
                                        format.push_str("{ERROR: ");
                                        format.push_str(err);
                                        format.push('}');
                                    }
                                }
                            }
                            format
                        }
                        None => "%+".to_owned(),
                    };

                    Chunk::Time(format)
                }
                "" => Chunk::Count(Count { count: 0, base: 0 }),
                "c" | "count" => {
                    if formatter.args.len() > 2 {
                        return Chunk::Error("expected at most two arguments".to_owned());
                    }
                    let mut count = Count { count: 0, base: 0 };
                    match formatter.args.get(0) {
                        Some(arg) => {
                            if arg.len() != 1 {
                                return Chunk::Error("invalid count".to_owned());
                            }
                            match arg[0] {
                                Piece::Text(ref z) => {
                                    if let Ok(c) = z.parse() {
                                        count.count = c;
                                    } else {
                                        return Chunk::Error(format!("invalid count `{}`", z));
                                    }
                                }
                                _ => return Chunk::Error("invalid count".to_owned()),
                            }
                        }
                        None => count.count = 0,
                    };
                    match formatter.args.get(1) {
                        Some(arg) => {
                            if arg.len() != 1 {
                                return Chunk::Error("invalid based".to_owned());
                            }
                            match arg[0] {
                                Piece::Text(ref z) => {
                                    if let Ok(b) = z.parse() {
                                        count.base = b;
                                    } else {
                                        return Chunk::Error(format!("invalid base `{}`", z));
                                    }
                                }
                                _ => return Chunk::Error("invalid base".to_owned()),
                            }
                        }
                        None => count.base = 0,
                    };
                    Chunk::Count(count)
                }
                name => Chunk::Error(format!("unknown formatter `{}`", name)),
            },
        }
    }
}

///! parse log path to Chunk
pub fn parse_to_chunk(pattern: &str) -> Vec<Chunk> {
    Parser::new(&pattern).map(From::from).collect()
}

#[cfg(test)]
mod tests {
    use super::{Chunk, Count};
    use crate::encode::pattern::parser::Parser;

    #[test]
    fn test_simple_nix_parse() {
        let file = "/some_dir/foo.log.";
        let pattern = format!("{}{{}}", file);
        let chunks: Vec<Chunk> = Parser::new(&pattern).map(From::from).collect();
        let text = chunks.get(0).unwrap();
        let count = chunks.get(1).unwrap();

        match text {
            Chunk::Text(str) => assert_eq!(str, file),
            _ => assert_eq!(true, false),
        }
        match count {
            Chunk::Count(c) => assert_eq!(*c, Count { base: 0, count: 0 }),
            _ => assert_eq!(true, false),
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_simple_win_parse() {
        let file = "c:\\some_dir\\foo.log.";
        let pattern = format!("{}{{}}", file);
        let chunks: Vec<Chunk> = Parser::new(&pattern).map(From::from).collect();
        let text = chunks.get(0).unwrap();
        let count = chunks.get(1).unwrap();

        match text {
            Chunk::Text(str) => assert_eq!(str, file),
            _ => assert_eq!(true, false),
        }
        match count {
            Chunk::Count(c) => assert_eq!(*c, Count { base: 0, count: 0 }),
            _ => assert_eq!(true, false),
        }
    }

    #[test]
    fn test_count_parse() {
        let file = "foo.log.";
        let pattern = format!("{}{{c(5)}}", file);
        let chunks: Vec<Chunk> = Parser::new(&pattern).map(From::from).collect();
        let text = chunks.get(0).unwrap();
        let count = chunks.get(1).unwrap();

        match text {
            Chunk::Text(str) => assert_eq!(str, file),
            _ => assert_eq!(true, false),
        }
        match count {
            Chunk::Count(c) => assert_eq!(*c, Count { base: 0, count: 5 }),
            _ => assert_eq!(true, false),
        }
    }

    #[test]
    fn test_date_parse() {
        let file = "foo.log.";
        let fmt = "%Y%m%d";
        let pattern = format!("{}{{d({})}}", file, fmt);
        let chunks: Vec<Chunk> = Parser::new(&pattern).map(From::from).collect();
        let text = chunks.get(0).unwrap();
        let time = chunks.get(1).unwrap();

        match text {
            Chunk::Text(str) => assert_eq!(str, file),
            _ => assert_eq!(true, false),
        }
        match time {
            Chunk::Time(t) => assert_eq!(t, fmt),
            _ => assert_eq!(true, false),
        }
    }

    #[test]
    fn test_unkown_parse() {
        let pattern = "foo.log.{x(unkown)}";
        let chunks: Vec<Chunk> = Parser::new(pattern).map(From::from).collect();
        let unkown = chunks.get(1).unwrap();
        match unkown {
            Chunk::Error(e) => assert_eq!(e, "unknown formatter `x`"),
            _ => assert_eq!(true, false),
        }
    }
}
