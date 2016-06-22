//! The syslog appender.

use log::LogRecord;
use std::error::Error;
use std::io::{self};
use std::net::{ToSocketAddrs, SocketAddr, TcpStream, UdpSocket};
use std::sync::Mutex;

use append::Append;
use encode::Encode;
use encode::pattern::PatternEncoder;
use encode::writer::SimpleWriter;

const DEFAULT_PROTOCOL: Protocol = Protocol::Udp;
const DEFAULT_ADDRESS: &'static str = "localhost:154";
const DEFAULT_PORT: u16 = 154;

/// Supported protocols.
pub enum Protocol {
	/// UDP
	Udp,
	/// TCP
	Tcp
}

/// Writers to syslog that utilize different protocols.
#[derive(Debug)]
enum SyslogWriter {
	Udp(Box<UdpSocket>, SocketAddr),
	Tcp(Mutex<SimpleWriter<TcpStream>>)
}

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

/// Appender that sends log messages to syslog.
#[derive(Debug)]
pub struct SyslogAppender {
	writer: SyslogWriter,
	encoder: Box<Encode>
}

/*
impl fmt::Debug for SyslogAppender {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let ds = fmt.debug_struct("SyslogAppender");
        match self.writer {
        	SyslogWriter::Udp(ref s, ref a) => ds.field("type", "udp".to_string()),
        	SyslogWriter::Tcp(ref s) => ds.field("type", "tcp".to_string())
        };
        ds.field("encoder", &self.encoder)
           .finish()
    }
}
*/
impl Append for SyslogAppender {
    fn append(&self, record: &LogRecord) -> Result<(), Box<Error>> {
		match self.writer {
			SyslogWriter::Udp(ref socket, ref addrs) => {
				let mut w = SimpleWriter(UdpWriter::new(socket, addrs));
				try!(self.encoder.encode(&mut w, record))
			},
			SyslogWriter::Tcp(ref tcp_w) => {
				let mut s = tcp_w.lock().unwrap();
				try!(self.encoder.encode(&mut *s, record))
			} 
		};
		// TODO: Flush if needed
		//try!(self.encoder.encode(&mut io_writer, record));
		//try!(io_writer.flush());
		Ok(())
    }
}

/// Builder for `SyslogAppender`.
pub struct SyslogAppenderBuilder<'a> {
	protocol: Protocol,
	encoder: Option<Box<Encode>>,
	addrs: &'a str
}

impl<'a> SyslogAppenderBuilder<'a> {
	/// Creates a `SyslogAppenderBuilder` for constructing new `SyslogAppender`.
	pub fn new() -> SyslogAppenderBuilder<'a> {
		SyslogAppenderBuilder {
			protocol: DEFAULT_PROTOCOL,
			addrs: DEFAULT_ADDRESS,
			encoder: None
		}
	}

	/// Sets network protocol for accessing syslog.
	/// 
	/// Defaults to UDP.
	pub fn protocol(mut self, p: Protocol) -> SyslogAppenderBuilder<'a> {
		self.protocol = p;
		self
	}

    /// Sets the output encoder for the `SyslogAppender`.
    pub fn encoder(mut self, encoder: Box<Encode>) -> SyslogAppenderBuilder<'a> {
        self.encoder = Some(encoder);
        self
    }

	/// Sets network address of syslog server.
	///
	/// Defaults to "localhost:154".
	pub fn addrs(mut self, addrs: &'a str) -> SyslogAppenderBuilder<'a> {
		self.addrs = addrs;
		self
	}

	/// Produces a `SyslogAppender` with parameters, supplied to the builder.
	pub fn finalize(self) -> io::Result<SyslogAppender> {
		let sa = norm_addrs(self.addrs);
		let a = sa.as_str();
		let writer = match self.protocol {
			Protocol::Udp => udp(a),
			Protocol::Tcp => tcp(a)
		};
		Ok(SyslogAppender {
			writer: writer,
			encoder: self.encoder.unwrap_or_else(|| Box::new(PatternEncoder::default()))
		})
	}
}

/// Creates writer for UDP protocol based on external host and port
fn udp<T: ToSocketAddrs>(rem: T) -> SyslogWriter {
	let socket = UdpSocket::bind("0.0.0.0:1234").unwrap();
	let rem_addrs = rem.to_socket_addrs().unwrap().next().unwrap();
	SyslogWriter::Udp(Box::new(socket), rem_addrs)
}

/// Creates writer for TCP protocol based on external host and port
fn tcp<T: ToSocketAddrs>(rem: T) -> SyslogWriter {
	let stream = TcpStream::connect(rem).unwrap();
	SyslogWriter::Tcp(Mutex::new(SimpleWriter(stream)))
}

/// Normalizes network address -- adds port if necessary 
fn norm_addrs(addrs: &str) -> String {
	let mut na = String::from(addrs);
	if !na.find(':').is_some() {
		na = format!("{}:{}", addrs, DEFAULT_PORT);
	}
	na
}

#[cfg(test)]
mod test {
	// use super::*;
	use super::norm_addrs;

/*	#[test]	
	fn test_udp_config() {
		let mut ap = SyslogAppenderBuilder::new()
			.protocol(Protocol::Udp)
			.addrs("10.211.55.6:154")
			.finalize()
			.unwrap();
	}*/

	#[test]
	fn norm_addrs_adds_default_port() {
		let na = norm_addrs("localhost");
		assert_eq!("localhost:154", na);
	}

	#[test]
	fn norm_addrs_doesnt_add_port_if_already_set() {
		let na = norm_addrs("localhost:1854");
		assert_eq!("localhost:1854", na);
	}
}



