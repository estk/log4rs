// cribbed to a large extent from libfmt_macros
use std::iter::Peekable;
use std::str::CharIndices;

pub enum Piece<'a> {
    Text(&'a str),
    Argument {
        formatter: Formatter<'a>,
    },
    Error(String),
}

pub struct Formatter<'a> {
    pub name: &'a str,
    pub arg: &'a str,
}

pub struct Parser<'a> {
    pattern: &'a str,
    it: Peekable<CharIndices<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(pattern: &'a str) -> Parser<'a> {
        Parser {
            pattern: pattern,
            it: pattern.char_indices().peekable(),
        }
    }

    fn consume(&mut self, ch: char) -> bool {
        match self.it.peek() {
            Some(&(_, c)) if c == ch => {
                self.it.next();
                true
            }
            _ => false
        }
    }

    fn argument(&mut self) -> Piece<'a> {
        match self.formatter() {
            Ok(formatter) => {
                Piece::Argument {
                    formatter: formatter,
                }
            }
            Err(err) => Piece::Error(err),
        }
    }

    fn formatter(&mut self) -> Result<Formatter<'a>, String> {
        Ok(Formatter {
            name: self.name(),
            arg: try!(self.arg()),
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

    fn arg(&mut self) -> Result<&'a str, String> {
        if !self.consume('(') {
            return Ok("");
        }

        let start = match self.it.next() {
            Some((_, ')')) => return Ok(""),
            Some((pos, _)) => pos,
            None => return Err("unclosed '('".to_owned()),
        };

        loop {
            match self.it.next() {
                Some((pos, ')')) => return Ok(&self.pattern[start..pos]),
                Some(_) => {}
                None => return Err("enclosed '('".to_owned()),
            }
        }
    }

    fn text(&mut self, start: usize) -> Piece<'a> {
        while let Some(&(pos, ch)) = self.it.peek() {
            match ch {
                '{' | '}' => return Piece::Text(&self.pattern[start..pos]),
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
            Some(&(pos, _)) => Some(self.text(pos)),
            None => None,
        }
    }
}

