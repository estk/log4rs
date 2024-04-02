// cribbed to a large extent from libfmt_macros
use std::{iter::Peekable, str::CharIndices};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Piece<'a> {
    Text(&'a str),
    Argument {
        formatter: Formatter<'a>,
        parameters: Parameters,
    },
    Error(String),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Formatter<'a> {
    pub name: &'a str,
    pub args: Vec<Vec<Piece<'a>>>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Parameters {
    pub fill: char,
    pub align: Alignment,
    pub min_width: Option<usize>,
    pub max_width: Option<usize>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Alignment {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct Parser<'a> {
    pattern: &'a str,
    it: Peekable<CharIndices<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(pattern: &'a str) -> Parser<'a> {
        Parser {
            pattern,
            it: pattern.char_indices().peekable(),
        }
    }

    fn consume(&mut self, ch: char) -> bool {
        match self.it.peek() {
            Some(&(_, c)) if c == ch => {
                self.it.next();
                true
            }
            _ => false,
        }
    }

    fn argument(&mut self) -> Piece<'a> {
        let formatter = match self.formatter() {
            Ok(formatter) => formatter,
            Err(err) => return Piece::Error(err),
        };

        Piece::Argument {
            formatter,
            parameters: self.parameters(),
        }
    }

    fn formatter(&mut self) -> Result<Formatter<'a>, String> {
        Ok(Formatter {
            name: self.name(),
            args: self.args()?,
        })
    }

    fn name(&mut self) -> &'a str {
        let start = match self.it.peek() {
            Some(&(pos, ch)) if ch.is_alphabetic() => {
                self.it.next();
                pos
            }
            _ => return "",
        };

        loop {
            match self.it.peek() {
                Some(&(_, ch)) if ch.is_alphanumeric() => {
                    self.it.next();
                }
                Some(&(end, _)) => return &self.pattern[start..end],
                None => return &self.pattern[start..],
            }
        }
    }

    fn args(&mut self) -> Result<Vec<Vec<Piece<'a>>>, String> {
        let mut args = vec![];
        while let Some(&(_, '(')) = self.it.peek() {
            args.push(self.arg()?);
        }
        Ok(args)
    }

    fn arg(&mut self) -> Result<Vec<Piece<'a>>, String> {
        if !self.consume('(') {
            return Ok(vec![]);
        }

        let mut arg = vec![];
        loop {
            if self.consume(')') {
                return Ok(arg);
            } else {
                match self.next() {
                    Some(piece) => arg.push(piece),
                    None => return Err("unclosed '('".to_owned()),
                }
            }
        }
    }

    fn parameters(&mut self) -> Parameters {
        let mut params = Parameters {
            fill: ' ',
            align: Alignment::Left,
            min_width: None,
            max_width: None,
        };

        if !self.consume(':') {
            return params;
        }

        if let Some(&(_, ch)) = self.it.peek() {
            match self.it.clone().nth(1) {
                Some((_, '<')) | Some((_, '>')) => {
                    self.it.next();
                    params.fill = ch;
                }
                _ => {}
            }
        }

        if self.consume('<') {
            params.align = Alignment::Left;
        } else if self.consume('>') {
            params.align = Alignment::Right;
        }

        if let Some(min_width) = self.integer() {
            params.min_width = Some(min_width);
        }

        if self.consume('.') {
            if let Some(max_width) = self.integer() {
                params.max_width = Some(max_width);
            }
        }

        params
    }

    fn integer(&mut self) -> Option<usize> {
        let mut cur = 0;
        let mut found = false;
        while let Some(&(_, ch)) = self.it.peek() {
            if let Some(digit) = ch.to_digit(10) {
                cur = cur * 10 + digit as usize;
                found = true;
                self.it.next();
            } else {
                break;
            }
        }

        if found {
            Some(cur)
        } else {
            None
        }
    }

    fn text(&mut self, start: usize) -> Piece<'a> {
        while let Some(&(pos, ch)) = self.it.peek() {
            match ch {
                '{' | '}' | '(' | ')' | '\\' => return Piece::Text(&self.pattern[start..pos]),
                _ => {
                    self.it.next();
                }
            }
        }
        Piece::Text(&self.pattern[start..])
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Piece<'a>;

    fn next(&mut self) -> Option<Piece<'a>> {
        match self.it.peek() {
            Some(&(_, '{')) => {
                self.it.next();
                if self.consume('{') {
                    Some(Piece::Text("{"))
                } else {
                    let piece = self.argument();
                    if self.consume('}') {
                        Some(piece)
                    } else {
                        for _ in &mut self.it {}
                        Some(Piece::Error("expected '}'".to_owned()))
                    }
                }
            }
            Some(&(_, '}')) => {
                self.it.next();
                if self.consume('}') {
                    Some(Piece::Text("}"))
                } else {
                    Some(Piece::Error("unmatched '}'".to_owned()))
                }
            }
            Some(&(_, '(')) => {
                self.it.next();
                if self.consume('(') {
                    Some(Piece::Text("("))
                } else {
                    Some(Piece::Error("unexpected '('".to_owned()))
                }
            }
            Some(&(_, ')')) => {
                self.it.next();
                if self.consume(')') {
                    Some(Piece::Text(")"))
                } else {
                    Some(Piece::Error("unexpected ')'".to_owned()))
                }
            }
            Some(&(_, '\\')) => {
                self.it.next();
                match self.it.peek() {
                    Some(&(_, '{')) => {
                        self.it.next();
                        Some(Piece::Text("{"))
                    }
                    Some(&(_, '}')) => {
                        self.it.next();
                        Some(Piece::Text("}"))
                    }
                    Some(&(_, '(')) => {
                        self.it.next();
                        Some(Piece::Text("("))
                    }
                    Some(&(_, ')')) => {
                        self.it.next();
                        Some(Piece::Text(")"))
                    }
                    Some(&(_, '\\')) => {
                        self.it.next();
                        Some(Piece::Text("\\"))
                    }
                    _ => Some(Piece::Error("unexpected '\\'".to_owned())),
                }
            }
            Some(&(pos, _)) => Some(self.text(pos)),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_arg_parser() {
        let pattern = "(%Y-%m-%dT%H:%M:%S%.6f";
        let mut parser = Parser::new(pattern);

        let arg = parser.arg();
        assert!(arg.is_err());

        let pattern = "(%Y-%m-%dT%H:%M:%S%.6f)";
        let mut parser = Parser::new(pattern);

        let arg = parser.arg();
        assert!(arg.is_ok());

        let pattern = "[{d(%Y-%m-%dT%H:%M:%S%.6f)} {h({l}):<5.5} {M}] {m}{n}";
        let mut parser = Parser::new(pattern);

        let arg = parser.arg();
        assert!(arg.is_ok());
        assert!(arg.unwrap().is_empty());
    }

    #[test]
    fn test_name() {
        // match up to first non alpha numberic
        let pattern = "test[";
        let mut parser = Parser::new(pattern);
        let name = parser.name();
        assert_eq!(name, "test");

        // match up to first non alpha numberic, so empty string
        let pattern = "[";
        let mut parser = Parser::new(pattern);
        let name = parser.name();
        assert_eq!(name, "");

        // match up to first non alpha numberic, so empty string
        let pattern = "test";
        let mut parser = Parser::new(pattern);
        let name = parser.name();
        assert_eq!(name, "test");
    }

    #[test]
    fn test_argument_invalid_and_valid() {
        let pattern = "(%Y-%m-%dT%H:%M:%S%.6f";
        let mut parser = Parser::new(pattern);

        let piece = parser.argument();
        assert!(match piece {
            Piece::Error(_) => true,
            _ => false,
        });

        let pattern = "[{d(%Y-%m-%dT%H:%M:%S%.6f)} {h({l}):<5.5} {M}] {m}{n}";
        let mut parser = Parser::new(pattern);

        let piece = parser.argument();
        assert!(match piece {
            Piece::Argument { .. } => true,
            _ => false,
        });
    }

    #[test]
    fn test_unmatched_bracket() {
        let pattern = "d}";
        let parser = Parser::new(pattern);
        let mut iter = parser.into_iter();

        // First parse the d
        assert!(match iter.next().unwrap() {
            Piece::Text { .. } => true,
            _ => false,
        });

        // Next try and parse the } but it's unmatched
        assert!(match iter.next().unwrap() {
            Piece::Error { .. } => true,
            _ => false,
        });
    }
}
