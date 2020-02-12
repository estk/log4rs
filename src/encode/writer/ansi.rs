//! The ANSI writer.
//!
//! Requires the `ansi_writer` feature.

use crate::cstd::io;
use crate::encode::{self, Color, Style};
use std::fmt;

#[cfg(feature = "async_fs")]
use async_trait::async_trait;
#[cfg(feature = "async_fs")]
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// An `encode::Write`r that wraps an `io::Write`r, emitting ANSI escape codes
/// for text style.
#[derive(Debug)]
pub struct AnsiWriter<W>(pub W);
impl<W> AnsiWriter<W> {
    #[cfg(feature = "async_fs")]
    fn get_mut_inner(self: Pin<&mut Self>) -> Pin<&mut W> {
        // This is okay because `field` is pinned when `self` is.
        unsafe { self.map_unchecked_mut(|s| &mut s.0) }
    }
    fn style_to_buf(&self, style: &Style) -> ([u8; 12], usize) {
        let mut buf = [0; 12];
        buf[0] = b'\x1b';
        buf[1] = b'[';
        buf[2] = b'0';
        let mut idx = 3;

        if let Some(text) = style.text {
            buf[idx] = b';';
            buf[idx + 1] = b'3';
            buf[idx + 2] = color_byte(text);
            idx += 3;
        }

        if let Some(background) = style.background {
            buf[idx] = b';';
            buf[idx + 1] = b'4';
            buf[idx + 2] = color_byte(background);
            idx += 3;
        }

        if let Some(intense) = style.intense {
            buf[idx] = b';';
            if intense {
                buf[idx + 1] = b'1';
                idx += 2;
            } else {
                buf[idx + 1] = b'2';
                buf[idx + 2] = b'2';
                idx += 3;
            }
        }
        buf[idx] = b'm';
        (buf, idx)
    }
}

#[cfg(feature = "async_fs")]
impl<W: io::Write> io::Write for AnsiWriter<W> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        self.get_mut_inner().poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        self.get_mut_inner().poll_flush(cx)
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        self.get_mut_inner().poll_close(cx)
    }
}
#[cfg(not(feature = "async_fs"))]
impl<W: io::Write> io::Write for AnsiWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }
    fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.0.write_fmt(fmt)
    }
}

#[cfg(feature = "async_fs")]
#[async_trait(?Send)]
impl<W: io::Write> encode::Write for AnsiWriter<W> {
    async fn set_style(self: Pin<&mut Self>, style: &Style) -> io::Result<()> {
        use async_std::io::prelude::WriteExt;
        let (buf, idx) = self.style_to_buf(style);
        self.get_mut_inner().write_all(&buf[..=idx]).await
    }
}
#[cfg(not(feature = "async_fs"))]
impl<W: io::Write> encode::Write for AnsiWriter<W> {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        let (buf, idx) = self.style_to_buf(style);
        self.0.write_all(&buf[..=idx])
    }
}

fn color_byte(c: Color) -> u8 {
    match c {
        Color::Black => b'0',
        Color::Red => b'1',
        Color::Green => b'2',
        Color::Yellow => b'3',
        Color::Blue => b'4',
        Color::Magenta => b'5',
        Color::Cyan => b'6',
        Color::White => b'7',
    }
}

#[cfg(test)]
mod test {
    use crate::cstd::io::{self, Write};

    use super::*;
    use crate::encode::Write as EncodeWrite;
    use crate::encode::{Color, Style};

    #[test]
    fn basic() {
        let stdout = io::stdout();
        let mut w = AnsiWriter(stdout.lock());

        w.write_all(b"normal ").unwrap();
        w.set_style(
            Style::new()
                .text(Color::Red)
                .background(Color::Blue)
                .intense(true),
        )
        .unwrap();
        w.write_all(b"styled").unwrap();
        w.set_style(Style::new().text(Color::Green)).unwrap();
        w.write_all(b" styled2").unwrap();
        w.set_style(&Style::new()).unwrap();
        w.write_all(b" normal\n").unwrap();
        w.flush().unwrap();
    }
}
