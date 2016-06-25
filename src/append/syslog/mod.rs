//! The syslog appender.
//!
//! # Features
//!
//! - TCP and UDP are supported as transport protocols.
//! - RFC 5424 and plain formats are supported.
//! - Usage of BOM marker in RFC 5424 messages might be switched off in configuration if not
//! properly displayed by syslog.
//!
//! # Limitations
//!
//! Since the syslog appender is a part of log4rs logging framework that utilizes the standard
//! `log` crate, only `ERROR`, `WARNING`, `INFO` and `DEBUG` facilities are supported.
//!
//! Unique message ID are not generated for now. `HOSTNAME`, `APP_NAME` and `PROCID` parameters
//! are not automatically computed, though the first two can be provided in configuration. `STRUCTURED-DATA`
//! is also not supported for now.
//!
//! # Configuration
//!
//! The syslog appender has default values for all its parameters so in the mimimal configuration you
//! can just drop an appender with `kind = syslog` into your config file. That will make the appender
//! send messages to the syslog on the same host via UDP in plain format.
//!
//! If you need more control on what the appender is doing, use these appender's parameters:
//!
//! - `protocol` &mdash; The transport protocol for sending messages to syslog. `"udp"` (default) and `"tcp"`
//! are supported.
//! - `address` &mdash; Network address of the host where the syslog is running. If a port number is not specified,
//! the standard port (514) will be used. The defaulf value is `"localhost:514"`.
//! - `max_len` &mdash; Maximum message length in bytes (all the headers included). Default is 2048.
//!
//! The default message format is `plain` which means that only the PRI part is added to a message that
//! you provide to the logger. If you want to use RFC 5424 format, you must provide `format` section
//! with `kind = rfc5424` to your appender.
//!
//! RFC 5425 format parameters:
//!
//! - `facility` &mdash; The `FACILITY` part of the message. Supported  values are `"kern"`, `"user"` (default), `"mail"`,
//! `"daemon"`, `"auth"`, `"syslog"`, `"lpr"`, `"news"`, `"uucp"`, `"cron"`, `"authpriv"`, `"ftp"`, `"ntp"`, `"logau"`, `"logalt"`, `"cron2"`,
//! `"local1"`, `"local2"`, `"local3"`, `"local4"`, `"local5"`, `"local6"`, `"local7"`.
//! - `hostname` &mdash; The `HOSTNAME` part of the message. Default is `NILVALUE` that resulst in symbol `'-'`.
//! - `app_name` &mdash; The `APP-NAME` part of the messages. Default is `NILVALUE` that resulst in symbol `'-'`.
//! - `bom` &mdash; Flag that can be used to switch BOM marker off in the resulting message. Default is `true`
//! (the marker is used).

#[cfg_attr(rustfmt, rustfmt_skip)]
mod serde;
pub mod consts;
pub mod plain;
pub mod rfc5424;

use log::LogRecord;
use serde::de;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, ErrorKind, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::Mutex;

use append::Append;
use append::syslog::serde::SyslogAppenderConfig;
use file::{Deserialize, Deserializers};
use serde_value::Value;

const DEFAULT_PROTOCOL: &'static str = "udp";
const DEFAULT_PORT: u16 = 514;
const DEFAULT_ADDRESS: &'static str = "localhost:514";
const DEFAULT_MAX_LENGTH: usize = 2048; // bytes

/// Writers to syslog that utilize different protocols.
#[derive(Debug)]
enum SyslogWriter {
	Udp(Box<UdpSocket>, SocketAddr),
	Tcp(Mutex<TcpStream>)
}

/// Syslog message format.
#[derive(Debug)]
pub enum MsgFormat {
    /// No formatting is applied.
    Plain(plain::Format),
    /// RFC 5424 format.
    Rfc5424(Box<rfc5424::Format>)
}

/*
/// Writer to UDP socket
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
	msg_format: MsgFormat,
	max_len: usize
	// encoder: Box<Encode>
}

impl Append for SyslogAppender {
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error>> {
		let message: String = match self.msg_format {
		    MsgFormat::Plain(ref fmt)    => fmt.apply(&record),
		    MsgFormat::Rfc5424(ref fmt) => fmt.apply(&record)
		};
		let mut bytes = message.as_bytes();
		if bytes.len() > self.max_len {
		    bytes = &bytes[0..self.max_len];
		}
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
	max_len: usize,
	msg_format: Option<MsgFormat>
	// encoder: Option<Box<Encode>>
}

impl SyslogAppenderBuilder {
	/// Creates a `SyslogAppenderBuilder` for constructing new `SyslogAppender`.
	pub fn new() -> SyslogAppenderBuilder {
		SyslogAppenderBuilder {
			protocol: DEFAULT_PROTOCOL.to_string(),
			addrs: DEFAULT_ADDRESS.to_string(),
			max_len: DEFAULT_MAX_LENGTH,
			msg_format: None
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
	/// Defaults to `Plain`.
	pub fn format(&mut self, mf: MsgFormat) -> &mut SyslogAppenderBuilder {
		self.msg_format = Some(mf);
		self
	}

    /// Sets the maximum length of a message in bytes. If a log message exceedes
    /// this size, it's truncated with not respect to UTF char boundaries.
    ///
    /// Defaults to 2048.
    pub fn max_len(&mut self, ml: usize) -> &mut SyslogAppenderBuilder {
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
		} else if self.protocol == "udp" {
		    writer = udp_writer(self.addrs.as_str());
		} else {
		   return Err(io::Error::new(ErrorKind::Other, format!("Unsupported syslog transport protocol {}", self.protocol).as_str()));
		}
		let appender = SyslogAppender {
			writer: writer,
			msg_format: self.msg_format.unwrap_or(MsgFormat::Plain(plain::Format::new())),
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

/// Stores information on format kind and its config parameters.
#[doc(hidden)]
pub struct FormatConf {
    /// Kind of the format.
    pub kind: String,
    /// Other contents to be parsed.
    pub config: Value,
}

impl de::Deserialize for FormatConf {
    fn deserialize<D>(d: &mut D) -> Result<FormatConf, D::Error>
        where D: de::Deserializer
    {
        let mut map = try!(BTreeMap::<Value, Value>::deserialize(d));

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => try!(kind.deserialize_into().map_err(|e| e.to_error())),
            None => return Err(de::Error::missing_field("kind")),
        };

        Ok(FormatConf {
            kind: kind,
            config: Value::Map(map),
        })
    }
}

/// Deserializer for `SyslogAppender`.
pub struct SyslogAppenderDeserializer;

impl Deserialize for SyslogAppenderDeserializer {
    type Trait = Append;

    fn deserialize(&self, config: Value, deserializers: &Deserializers) -> Result<Box<Append>, Box<Error>> {
        let config = try!(config.deserialize_into::<SyslogAppenderConfig>());
        let mut builder = SyslogAppenderBuilder::new();
        if let Some(prot) = config.protocol {
        	builder.protocol(prot);
        }
        if let Some(addrs) = config.address {
        	builder.address(addrs);
        }
        if let Some(ml) = config.max_len {
            builder.max_len(ml);
        }
        if let Some(format) = config.format {
            if format.kind == "rfc5424" {
                builder.format(MsgFormat::Rfc5424(try!(deserializers.deserialize("format", &format.kind, format.config))));
            } else if format.kind == "plain" {
                builder.format(MsgFormat::Plain(plain::Format::new()));
            } else {
    		    return Err(Box::new(io::Error::new(ErrorKind::Other, format!("Unsupported syslog message format {}", format.kind).as_str())));
            }
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
