/// This module implements low-level communication with the target device.
/// It does not actually understand protocol details, but only provides dumb bidirectional pipes.
pub mod net_bind;
pub mod net_connect;
pub mod usb;

pub use std::io::Result;

/// This trait implements an interface that allows for decoupling
/// the transport of bytes to and from the target from the actual Odin protocol implementation.
pub trait Communicator {
    /// Send the entire buffer to the device, blocking until it's sent or an error occurs.
    /// Will retry send if the underlying medium supports it.
    ///
    /// Zero-length data handling is implementation-defined. Some implementations may send an empty
    /// lower-level transfer, while others may do nothing at all.
    fn send(&mut self, data: &[u8]) -> Result<()>;
    /// Receive exactly the specified amount of data from the device.
    /// Blocks until that much data could be collected or an error occurs.
    fn recv_exact(&mut self, how_much: usize) -> Result<Vec<u8>>;
}
