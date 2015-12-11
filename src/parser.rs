use nom::eof;
use pattern::Error;
use ErrorInternals;
use std::str;
use time;

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
