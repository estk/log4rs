//! The syslog appender.

#[cfg_attr(rustfmt, rustfmt_skip)]
mod serde;
mod rfc5424;

use log::{LogLevel, LogRecord};
use serde_value::Value;
use std::error::Error;
use std::io::{self, Write};
use std::net::{ToSocketAddrs, SocketAddr, TcpStream, UdpSocket};
use std::sync::Mutex;

use append::Append;
use append::syslog::serde::SyslogAppenderConfig;
use file::{Deserialize, Deserializers};

const DEFAULT_PROTOCOL: &'static str = "udp";
const DEFAULT_FORMAT: &'static str = "plain";
const DEFAULT_PORT: u16 = 514;
const DEFAULT_ADDRESS: &'static str = "localhost:514";
const DEFAULT_MAX_LENGTH: u16 = 2048; // bytes

/// Writers to syslog that utilize different protocols.
#[derive(Debug)]
enum SyslogWriter {
	Udp(Box<UdpSocket>, SocketAddr),
	Tcp(Mutex<TcpStream>)
}

#[derive(Debug)]
enum Format {
    Plain,
    RFC_5424(rfc5424::Format)
}

/// Writer to UDP socket
/*
struct UdpWriter<'a> {
	socket: &'a UdpSocket,
	addrs: &'a SocketAddr
}

impl<'a> UdpWriter<'a> {
	pub fn new(socket: &'a UdpSocket, addrs: &'a SocketAddr) -> UdpWriter<'a> {
		UdpWriter {
			socket: socket,
			addrs: addrs
		}
	}
}

impl<'a> io::Write for UdpWriter<'a> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.socket.send_to(buf, self.addrs)
	}

    fn flush(&mut self) -> io::Result<()> {
    	Ok(())
    }
}
*/

/// Appender that sends log messages to syslog.
#[derive(Debug)]
pub struct SyslogAppender {
	writer: SyslogWriter,
	format: Format,
	max_len: u16
	// encoder: Box<Encode>
}

impl Append for SyslogAppender {
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error>> {
		let message: String = match self.format {
		    Format::Plain            => format!("{}\n", record.args()),
		    Format::RFC_5424(ref fmt) => fmt.apply(&record)
		};
		let bytes = message.as_bytes();
		match self.writer {
			SyslogWriter::Udp(ref socket, ref addrs) => {
				try!(socket.send_to(&bytes, addrs));
				// let mut w = SimpleWriter(BufWriter::with_capacity(1024, UdpWriter::new(socket, addrs)));
				// try!(self.encoder.encode(&mut w, record))
			},
			SyslogWriter::Tcp(ref stream_w) => {
				let mut stream = stream_w.lock().unwrap();
				try!(stream.write(bytes));
				// TODO: broken pipe recovery: Broken pipe (os error 32)
				// try!(self.encoder.encode(&mut *s, record))
				// try!(s.flush())
			}
		};
		Ok(())
    }
}

/// Builder for `SyslogAppender`.
pub struct SyslogAppenderBuilder {
	protocol: String,
	addrs: String,
	max_len: u16,
	format: String
	// encoder: Option<Box<Encode>>
}

impl SyslogAppenderBuilder {
	/// Creates a `SyslogAppenderBuilder` for constructing new `SyslogAppender`.
	pub fn new() -> SyslogAppenderBuilder {
		SyslogAppenderBuilder {
			protocol: DEFAULT_PROTOCOL.to_string(),
			addrs: DEFAULT_ADDRESS.to_string(),
			max_len: DEFAULT_MAX_LENGTH,
			format: DEFAULT_FORMAT.to_string()
			// encoder: None
		}
	}

	/// Sets network protocol for accessing syslog.
	/// 
	/// Defaults to "udp".
	pub fn protocol(& mut self, p: String) -> &mut SyslogAppenderBuilder {
		self.protocol = p.to_lowercase();
		self
	}

	/// Sets network address of syslog server.
	///
	/// Defaults to "localhost:514".
	pub fn address(&mut self, addrs: String) -> &mut SyslogAppenderBuilder {
		self.addrs = addrs;
		self
	}

	/// Sets type of log message formatter.
	///
	/// Defaults to `plain`.
	pub fn format(&mut self, f: String) -> &mut SyslogAppenderBuilder {
		self.format = f.to_lowercase();
		self
	}

    /// Sets the maximum length of a message in bytes. If a log message exceedes
    /// this size, it's truncated with not respect to UTF char boundaries.
    ///
    /// Defaults to 2048.
    pub fn max_len(&mut self, ml: u16) -> &mut SyslogAppenderBuilder {
		self.max_len = ml;
		self
	}

    // Sets the output encoder for the `SyslogAppender`.
    // pub fn encoder(&mut self, encoder: Box<Encode>) -> &mut SyslogAppenderBuilder {
    //    self.encoder = Some(encoder);
    //    self
    //}

	/// Produces a `SyslogAppender` with parameters, supplied to the builder.
	pub fn finalize(mut self) -> io::Result<SyslogAppender> {
		norm_addrs(&mut self.addrs);
		let writer;
		if self.protocol == "tcp" {
		    writer = tcp_writer(self.addrs.as_str());
		} else {
		    // TODO: Error if not udp
		    writer = udp_writer(self.addrs.as_str());
		}
		let format;
		if self.format == "rfc5424" {
		    format = Format::RFC_5424(rfc5424::Format::default())
		} else {
		    // TODO: Error if not plain
		    format = Format::Plain;
		}
		let appender = SyslogAppender {
			writer: writer,
			format: format,
			max_len: self.max_len
			// encoder: self.encoder.unwrap_or_else(|| Box::new(PatternEncoder::default()))
		};
		Ok(appender)
	}
}

/// Normalizes network address -- adds port if necessary 
fn norm_addrs(addrs: &mut String) {
	if !addrs.find(':').is_some() {
		addrs.push(':');
		addrs.push_str(&DEFAULT_PORT.to_string())
	}
}

/// Creates writer for UDP protocol based on external host and port
fn udp_writer<T: ToSocketAddrs>(rem: T) -> SyslogWriter {
	let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
	let rem_addrs = rem.to_socket_addrs().unwrap().next().unwrap();
	SyslogWriter::Udp(Box::new(socket), rem_addrs)
}

/// Creates writer for TCP protocol based on external host and port
fn tcp_writer<T: ToSocketAddrs>(rem: T) -> SyslogWriter {
	let stream = TcpStream::connect(rem).unwrap();
	SyslogWriter::Tcp(Mutex::new(stream))
}

/// Deserializer for `SyslogAppender`.
pub struct SyslogAppenderDeserializer;

impl Deserialize for SyslogAppenderDeserializer {
    type Trait = Append;

    fn deserialize(&self, config: Value, _: &Deserializers) -> Result<Box<Append>, Box<Error>> {
        let config = try!(config.deserialize_into::<SyslogAppenderConfig>());
        let mut builder = SyslogAppenderBuilder::new();
        if let Some(prot) = config.protocol {
        	builder.protocol(prot);
        }
        if let Some(addrs) = config.address {
        	builder.address(addrs);
        }
        if let Some(fmt) = config.format {
        	builder.format(fmt);
        }
        if let Some(ml) = config.max_len {
            builder.max_len(ml);
        }
        // if let Some(encoder) = config.encoder {
        //   builder.encoder(try!(deserializers.deserialize("encoder",
        //                                                       &encoder.kind,
        //                                                       encoder.config)));
        //}
        Ok(Box::new(try!(builder.finalize())))
    }
}


#[cfg(test)]
mod test {
	use super::norm_addrs;

	#[test]
	fn norm_addrs_adds_default_port() {
		let mut addr = "localhost".to_string();
		norm_addrs(&mut addr);
		assert_eq!(addr.as_str(), "localhost:514");
	}

	#[test]
	fn norm_addrs_doesnt_add_port_if_already_set() {
		let mut addr = "localhost:5124".to_string();
		norm_addrs(&mut addr);
		assert_eq!(addr.as_str(), "localhost:5124");
	}
}



