use super::Communicator;

use std::io::{Read, Result as IOResult, Write};
use std::net::TcpStream;

/// Connector manages the TCP connection to the target.
pub struct Connection {
    s: TcpStream,
}

impl Connection {
    /// Establishes a new connection to the target.
    pub fn new(target_ip: &str, port: u16) -> Connection {
        let s = TcpStream::connect(format!("{}:{}", target_ip, port)).unwrap();

        log::debug!(target: "NET", "Connected");
        return Connection { s };
    }
}

impl Communicator for Connection {
    /// Sends the given data to the device.
    /// Blocks until all data could be sent or an error occurs.
    fn send(&mut self, data: &[u8]) -> IOResult<()> {
        log::trace!(target: "NET", "Send: {:?}", data);
        return self.s.write_all(data);
    }

    fn recv_exact(&mut self, how_much: usize) -> IOResult<Vec<u8>> {
        let mut buf = vec![];
        buf.resize(how_much, 0);
        self.s.read_exact(&mut buf)?;

        log::trace!(target: "NET", "Recv: {:?}", buf);
        return Ok(buf);
    }
}
