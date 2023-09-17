use super::*;

use std::io::{Read, Result as IOResult, Write};
use std::net::{TcpListener, TcpStream};

/// Listener accepts new wireless AP ODIN mode connections.
pub struct Listener {
    l: TcpListener,
}

impl Listener {
    /// Create a new listener listening on the given port, on all interfaces.
    /// IPv4 only.
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
        stream
            .set_read_timeout(Some(super::DEFAULT_TIMEOUT))
            .unwrap();
        stream
            .set_write_timeout(Some(super::DEFAULT_TIMEOUT))
            .unwrap();
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
        log::trace!(target: "NET", "Send: {}", format_data_buf(data));
        return self.s.write_all(data);
    }

    fn recv_exact(&mut self, how_much: usize) -> IOResult<Vec<u8>> {
        let mut buf = vec![];
        buf.resize(how_much, 0);
        self.s.read_exact(&mut buf)?;

        log::trace!(target: "NET", "Recv exact: {}", format_data_buf(&buf));
        return Ok(buf);
    }

    #[allow(clippy::read_zero_byte_vec)] // Handled, but clippy is too stupid to notice
    fn recv(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![];
        let bytes_read = self.s.read(&mut buf)?;
        log::trace!(target: "NET", "Recv nonblocking: {}", format_data_buf(&buf));
        // Should probably never happen
        if bytes_read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Read zero bytes",
            ));
        }
        return Ok(buf);
    }

    fn set_timeout(&mut self, timeout: Duration) {
        log::info!(target: "NET", "Setting timeout: {timeout:?}");
        self.s
            .set_read_timeout(Some(super::DEFAULT_TIMEOUT))
            .unwrap();
        self.s
            .set_write_timeout(Some(super::DEFAULT_TIMEOUT))
            .unwrap();
    }
}
