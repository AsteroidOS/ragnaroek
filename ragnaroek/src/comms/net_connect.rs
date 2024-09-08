use super::*;

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
        s.set_read_timeout(Some(super::DEFAULT_TIMEOUT)).unwrap();
        s.set_write_timeout(Some(super::DEFAULT_TIMEOUT)).unwrap();
        return Connection { s };
    }
}

impl Communicator for Connection {
    /// Sends the given data to the device.
    /// Blocks until all data could be sent or an error occurs.
    fn send(&mut self, data: &[u8]) -> IOResult<()> {
        log::trace!(target: "NET", "Send: {}", format_data_buf(data));
        return self.s.write_all(data);
    }

    fn recv_exact(&mut self, how_much: usize) -> IOResult<Vec<u8>> {
        let mut buf = vec![0; how_much];
        self.s.read_exact(&mut buf)?;

        log::trace!(target: "NET", "Recv: {}",format_data_buf(&buf));
        return Ok(buf);
    }

    #[allow(clippy::read_zero_byte_vec)] // Handled, but clippy is too stupid to notice
    fn recv(&mut self) -> IOResult<Vec<u8>> {
        let mut buf = vec![];
        let bytes_read = self.s.read(&mut buf)?;
        // Should probably never happen
        if bytes_read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Read zero bytes",
            ));
        }
        log::trace!(target: "NET", "Recv nonblocking: {}", format_data_buf(&buf));
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
