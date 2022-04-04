use super::Communicator;

use std::io::{Read, Result as IOResult, Write};
use std::net::{TcpListener, TcpStream};

/// Listener accepts new wireless AP ODIN mode connections.
pub struct Listener {
    l: TcpListener,
}

impl Listener {
    pub fn new(port: u16) -> Listener {
        // All currently known devices do not use IPv6
        let l = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();

        log::debug!(target: "NET", "Listening");
        return Listener { l };
    }

    /// Blocks until a device is connected.
    /// Returns a `Connection` once this happens.
    pub fn accept(&mut self) -> IOResult<Connection> {
        let (stream, _) = self.l.accept()?;

        log::debug!(target: "NET", "Accepted");
        return Ok(Connection { s: stream });
    }
}

/// `Connection` implements a wireless ODIN mode connection.
/// You can obtain this by calling `accept()` on a `Listener`.
pub struct Connection {
    s: TcpStream,
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
