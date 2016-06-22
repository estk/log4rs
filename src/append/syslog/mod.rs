//! The syslog appender.

#[cfg_attr(rustfmt, rustfmt_skip)]
mod serde;

use log::LogRecord;
use serde_value::Value;
use std::error::Error;
use std::io::{self, Write};
use std::net::{ToSocketAddrs, SocketAddr, TcpStream, UdpSocket};
use std::sync::Mutex;

use append::Append;
use file::{Deserialize, Deserializers};
use append::syslog::serde::SyslogAppenderConfig;

const DEFAULT_PROTOCOL: Protocol = Protocol::Udp;
const DEFAULT_ADDRESS: &'static str = "localhost:514";
const DEFAULT_PORT: u16 = 514;
const DEFAULT_HEADER: Header = Header::None;

/// Supported protocols.
pub enum Protocol {
	/// UDP
	Udp,
	/// TCP
	Tcp
}

/// Log messages header types.
#[derive(Debug)]
pub enum Header {
	/// No header will be added to messages.
	None,
	/// RFC 5424 headers.
	RFC5424
}

/// Writers to syslog that utilize different protocols.
#[derive(Debug)]
enum SyslogWriter {
	Udp(Box<UdpSocket>, SocketAddr),
	Tcp(Mutex<TcpStream>)
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
	header: Header
	// encoder: Box<Encode>
}

impl Append for SyslogAppender {
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error>> {
		let hdr = match self.header {
			Header::None => "".to_string(),
			Header::RFC5424 => SyslogAppender::header_rfc_5424()
		};
		println!("{}", hdr);
		match self.writer {
			SyslogWriter::Udp(ref socket, ref addrs) => {
				let message = &(format!("{}{}", hdr, record.args()));
				let bytes = message.as_bytes();
				try!(socket.send_to(&bytes, addrs));
				//let mut w = SimpleWriter(BufWriter::with_capacity(1024, UdpWriter::new(socket, addrs)));
				//try!(self.encoder.encode(&mut w, record))
			},
			SyslogWriter::Tcp(ref stream_w) => {
				let message = &(format!("{}{}\n", "tcp", record.args()));
				let bytes = message.as_bytes();
				let mut stream = stream_w.lock().unwrap();
				try!(stream.write(bytes));
				//try!(self.encoder.encode(&mut *s, record))
				//try!(s.flush())
			}
		};
		Ok(())
    }
}

impl SyslogAppender {
	fn header_rfc_5424() -> String {
		"RFC 5424 ".to_string()
	}
}

/// Builder for `SyslogAppender`.
pub struct SyslogAppenderBuilder {
	protocol: Protocol,
	header: Header,
	// encoder: Option<Box<Encode>>,
	addrs: String
}

impl SyslogAppenderBuilder {
	/// Creates a `SyslogAppenderBuilder` for constructing new `SyslogAppender`.
	pub fn new() -> SyslogAppenderBuilder {
		SyslogAppenderBuilder {
			protocol: DEFAULT_PROTOCOL,
			addrs: DEFAULT_ADDRESS.to_string(),
			header: DEFAULT_HEADER,
			// encoder: None
		}
	}

	/// Sets network protocol for accessing syslog.
	/// 
	/// Defaults to UDP.
	pub fn protocol(&mut self, p: Protocol) -> &mut SyslogAppenderBuilder {
		self.protocol = p;
		self
	}

    // Sets the output encoder for the `SyslogAppender`.
    // pub fn encoder(&mut self, encoder: Box<Encode>) -> &mut SyslogAppenderBuilder {
    //    self.encoder = Some(encoder);
    //    self
    //}

	/// Sets network address of syslog server.
	///
	/// Defaults to "localhost:514".
	pub fn addrs(&mut self, addrs: String) -> &mut SyslogAppenderBuilder {
		self.addrs = addrs;
		self
	}

	/// Sets type of log message headers.
	///
	/// Defaults to `None`.
	pub fn header(&mut self, h: Header) -> &mut SyslogAppenderBuilder {
		self.header = h;
		self
	}

	/// Produces a `SyslogAppender` with parameters, supplied to the builder.
	pub fn finalize(mut self) -> io::Result<SyslogAppender> {
		SyslogAppenderBuilder::norm_addrs(&mut self.addrs);
		let writer = match self.protocol {
			Protocol::Udp => SyslogAppenderBuilder::udp(self.addrs.as_str()),
			Protocol::Tcp => SyslogAppenderBuilder::tcp(self.addrs.as_str())
		};
		Ok(SyslogAppender {
			writer: writer,
			header: self.header
			// encoder: self.encoder.unwrap_or_else(|| Box::new(PatternEncoder::default()))
		})
	}

	/// Creates writer for UDP protocol based on external host and port
	fn udp<T: ToSocketAddrs>(rem: T) -> SyslogWriter {
		let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
		let rem_addrs = rem.to_socket_addrs().unwrap().next().unwrap();
		SyslogWriter::Udp(Box::new(socket), rem_addrs)
	}

	/// Creates writer for TCP protocol based on external host and port
	fn tcp<T: ToSocketAddrs>(rem: T) -> SyslogWriter {
		let stream = TcpStream::connect(rem).unwrap();
		SyslogWriter::Tcp(Mutex::new(stream))
	}
	
	/// Normalizes network address -- adds port if necessary 
	fn norm_addrs(addrs: &mut String) {
		if !addrs.find(':').is_some() {
			addrs.push(':');
			addrs.push_str(&DEFAULT_PORT.to_string())
		}
	}
}

/// Deserializer for `SyslogAppender`.
pub struct SyslogAppenderDeserializer;

impl Deserialize for SyslogAppenderDeserializer {
    type Trait = Append;

    fn deserialize(&self, config: Value, _: &Deserializers) -> Result<Box<Append>, Box<Error>> {
        let config = try!(config.deserialize_into::<SyslogAppenderConfig>());
        let mut builder = SyslogAppenderBuilder::new();
        if let Some(prot) = config.protocol {
        	if prot == "udp" {
        		builder.protocol(Protocol::Udp);
        	} else if prot == "tcp" {
        		builder.protocol(Protocol::Tcp);
        	} else {
        		// TODO: Throw error
        	}
        }
        if let Some(hdr) = config.header {
        	if hdr == "none" {
        		builder.header(Header::None);
        	} else if hdr == "rfc5424" {
        		builder.header(Header::RFC5424);
        	} else {
        		// TODO: Throw error
        	}
        }
        if let Some(addrs) = config.address {
        	builder.addrs(addrs);
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
	use super::SyslogAppenderBuilder;

	#[test]
	fn norm_addrs_adds_default_port() {
		let mut addr = "localhost".to_string();
		SyslogAppenderBuilder::norm_addrs(&mut addr);
		assert_eq!("localhost:514", addr.as_str());
	}

	#[test]
	fn norm_addrs_doesnt_add_port_if_already_set() {
		let mut addr = "localhost:5124".to_string();
		SyslogAppenderBuilder::norm_addrs(&mut addr);
		assert_eq!("localhost:5124", addr.as_str());
	}
}



