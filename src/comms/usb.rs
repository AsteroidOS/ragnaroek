/// These are taken from Heimdall, may not be exhaustive
const SAMSUNG_VID: u16 = 0x04E8;
const VALID_PIDS: [u16; 3] = [0x6601, 0x685D, 0x68C3];
/// USB class that the desired configuration has (USB communications device)
const USB_CLASS_CDC_DATA: u8 = 0x0A;

use rusb::{Device, Direction, GlobalContext};

use super::Communicator;

use std::io::Result as IOResult;
use std::time::Duration;

/// `Connection` implements a USB ODIN mode connection.
pub struct Connection {
    handle: rusb::DeviceHandle<GlobalContext>,
    send_endpoint: u8,
    recv_endpoint: u8,
}

impl Connection {
    /// Establish a new connection to the first viable USB device.
    /// Returns an error if no suitable device could be found.
    pub fn establish() -> IOResult<Connection> {
        // FIXME: Error handling
        for dev in rusb::devices().unwrap().iter() {
            let desc = dev.device_descriptor().unwrap();
            if desc.vendor_id() == SAMSUNG_VID && VALID_PIDS.contains(&desc.product_id()) {
                let mut handle = dev.open().unwrap();
                let _ = handle.set_auto_detach_kernel_driver(true);
                // FIXME: Is this always the interface we want?
                handle.claim_interface(1).unwrap();
                // Find endpoint suitable for sending data to the device
                let send_endpoint = find_endpoint(&dev, Direction::Out)
                    .expect("Could not find suitable endpoint for sending data to the USB device");
                // Find endpoint suitable for receiving data from the device
                let recv_endpoint = find_endpoint(&dev, Direction::In).expect(
                    "Could not find suitable endpoint for receiving data from the USB device",
                );
                handle.set_alternate_setting(1, 0).unwrap();

                return Ok(Connection {
                    handle,
                    send_endpoint,
                    recv_endpoint,
                });
            }
        }
        panic!("No suitable USB device found!");
    }
}

/// Finds an endpoint for the given direction, if any.
fn find_endpoint(dev: &Device<GlobalContext>, direction: Direction) -> Option<u8> {
    // Walk in the order configuration descriptor -> interface descriptor -> endpoint descriptor
    let conf = dev.active_config_descriptor().unwrap();
    for iface in conf.interfaces() {
        for descr in iface
            .descriptors()
            .filter(|d| d.class_code() == USB_CLASS_CDC_DATA)
        {
            for endpoint in descr.endpoint_descriptors() {
                if endpoint.direction() == direction {
                    // TODO: Re-read Heimdall code more carefully to figure out whether this is the only criterion
                    return Some(endpoint.number());
                }
            }
        }
    }
    return None;
}

impl Communicator for Connection {
    /// Sends the given data to the device.
    /// Blocks until all data could be sent or an error occurs.
    fn send(&mut self, data: &[u8]) -> IOResult<()> {
        self.handle
            .write_bulk(self.send_endpoint, data, Duration::from_secs(1))
            .unwrap();
        return Ok(());
    }

    fn recv_exact(&mut self, how_much: usize) -> IOResult<Vec<u8>> {
        let mut buf = vec![];
        buf.resize(how_much, 0);
        match self
            .handle
            .read_bulk(self.recv_endpoint, &mut buf, Duration::from_secs(1))
        {
            // FIXME: Should read repeatedly until desired data volume is reached instead
            Ok(read) => buf.resize(read, 0),
            Err(e) => panic!("{}", e),
        }
        return Ok(buf);
    }
}
