/// This module implements low-level communication with the target device.
/// It does not actually understand protocol details, but only provides dumb bidirectional pipes.
pub mod net_bind;
pub mod net_connect;
pub mod usb;

pub use std::io::Result;

/// This trait implements an interface that allows for decoupling
/// the transport of bytes to and from the target from the actual Odin protocol implementation.
pub trait Communicator: Send {
    /// Send the entire buffer to the device, blocking until it's sent or an error occurs.
    /// Will retry send if the underlying medium supports it.
    ///
    /// Zero-length data handling is implementation-defined. Some implementations may send an empty
    /// lower-level transfer, while others may do nothing at all.
    fn send(&mut self, data: &[u8]) -> Result<()>;
    /// Receive exactly the specified amount of data from the device.
    /// Blocks until that much data could be collected or an error occurs.
    fn recv_exact(&mut self, how_much: usize) -> Result<Vec<u8>>;
    /// Receive however much data is waiting to be read. Returned data may be empty.
    /// Does not block.
    fn recv(&mut self) -> Result<Vec<u8>>;
}

/// Helper feature for debug logging
fn format_data_buf(data: &[u8]) -> String {
    let mut s = String::from("[");
    // Cut trailing zeroes
    let mut vec: Vec<u8> = data
        .into_iter()
        .rev()
        .skip_while(|&x| *x == 0)
        .map(|x| *x)
        .collect();
    vec = vec.into_iter().rev().collect();
    let num_zeroes = data.len() - vec.len();

    for b in vec {
        s.push_str(&format!("0x{:X}, ", b));
    }
    if num_zeroes > 0 {
        s.push_str(&format!("<{} trailing 0's cut>", num_zeroes));
    }
    s.push(']');
    return s;
}
