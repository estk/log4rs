use nom::eof;
use std::fmt::{self, Write};
use std::str;
use time;

use encode::pattern::Error;
use ErrorInternals;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TimeFmt {
    Rfc3339,
    Str(String),
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Chunk {
    Text(String),
    Time(TimeFmt),
    Level,
    Message,
    Module,
    File,
    Line,
    Thread,
    Target,
}

impl fmt::Display for Chunk {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Chunk::Text(ref text) => {
                if text.contains('%') {
                    for ch in text.chars() {
                        if ch == '%' {
                            try!(fmt.write_str("%%"));
                        } else {
                            try!(fmt.write_char(ch));
                        }
                    }
                    Ok(())
                } else {
                    fmt.write_str(text)
                }
            }
            Chunk::Time(TimeFmt::Rfc3339) => fmt.write_str("%d"),
            Chunk::Time(TimeFmt::Str(ref s)) => write!(fmt, "%d{{{}}}", s),
            Chunk::Level => fmt.write_str("%l"),
            Chunk::Message => fmt.write_str("%m"),
            Chunk::Module => fmt.write_str("%M"),
            Chunk::File => fmt.write_str("%f"),
            Chunk::Line => fmt.write_str("%L"),
            Chunk::Thread => fmt.write_str("%T"),
            Chunk::Target => fmt.write_str("%t"),
        }
    }
}


named!(pub parse_pattern(&[u8]) -> Vec<Chunk>,
    chain!(
        result: many0!(
            alt!(
                escaped_percent |
                time |
                level |
                message |
                module |
                file |
                line |
                thread |
                target |
                regular_text
            )
        ) ~
        eof,
        || result
    )
);

named!(regular_text<Chunk>,
    chain!(
        text: map_res!(take_until!("%"), str::from_utf8),
        || Chunk::Text(text.into())
    )
);

named!(level<Chunk>,
    chain!(
        tag!("%l"),
        || Chunk::Level
    )
);

named!(message<Chunk>,
    chain!(
        tag!("%m"),
        || Chunk::Message
    )
);

named!(module<Chunk>,
    chain!(
        tag!("%M"),
        || Chunk::Module
    )
);

named!(file<Chunk>,
    chain!(
        tag!("%f"),
        || Chunk::File
    )
);

named!(line<Chunk>,
    chain!(
        tag!("%L"),
        || Chunk::Line
    )
);

named!(thread<Chunk>,
    chain!(
        tag!("%T"),
        || Chunk::Thread
    )
);

named!(target<Chunk>,
    chain!(
        tag!("%t"),
        || Chunk::Target
    )
);

named!(escaped_percent<Chunk>,
    chain!(
        tag!("%%"),
        || Chunk::Text("%".into())
    )
);

named!(time<Chunk>,
    chain!(
        tag!("%d") ~
        timefmt: timefmt,
        || Chunk::Time(timefmt)
    )
);

named!(timefmt<TimeFmt>,
    alt!(
        map_res!(delimited!(tag!("{"), timefmt_string, tag!("}")), check_timefmt_error) => { |res: String| TimeFmt::Str(res) }
        | take!(0)                                    => { |_| TimeFmt::Rfc3339 }
    )
);

named!(timefmt_string<String>,
    chain!(
        format: map_res!(take_until!("}"), str::from_utf8),
        || format.into())
);

fn check_timefmt_error(fmt: String) -> Result<String, Error> {
    if let Err(err) = time::now().strftime(&*fmt) {
        Err(Error::new(err.to_string()))
    } else {
        Ok(fmt)
    }
}
