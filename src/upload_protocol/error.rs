/// Error caused by the device violating our assumptions about
/// how a device in upload mode ought to behave.
#[derive(Clone, Copy, Debug)]
pub enum UploadProtocolError {
    /// Device did not send an acknowledgment packet.
    MissingAck,
    /// Start address occured before end address.
    /// First value is given start address, second given end address.
    EndAddrBeforeStartAddr(u64, u64),
}
