/// These are taken from Heimdall, may not be exhaustive
const SAMSUNG_VID: u16 = 0x04E8;
const VALID_PIDS: [u16; 3] = [0x6601, 0x685D, 0x68C3];
/// USB class that the desired configuration has (USB communications device)
const USB_CLASS_CDC_DATA: u8 = 0x0A;

use super::Communicator;
use rusb::{Device, Direction, GlobalContext, InterfaceDescriptor};

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
        // Find the device
        // TODO: Handle multi-device
        let dev: Option<Device<GlobalContext>> = rusb::devices()
            .unwrap()
            .iter()
            .filter(|dev| {
                let desc = dev.device_descriptor().unwrap();
                desc.vendor_id() == SAMSUNG_VID && VALID_PIDS.contains(&desc.product_id())
            })
            .next();
        let dev = dev.expect("Failed to find supported USB device!");

        let mut handle = dev.open().unwrap();
        // Not supported on macOS, ignore the error for now.
        let _ = handle.set_auto_detach_kernel_driver(true);
        // Find endpoint suitable for sending data to the device
        let (input, output, interface_num) = find_endpoints(&dev)
            .expect("Could not find suitable endpoint for sending data to the USB device");
        // Find endpoint suitable for receiving data from the device
        handle.claim_interface(interface_num).unwrap();
        handle.set_alternate_setting(interface_num, 0).unwrap();

        log::debug!(target: "USB", "Connected");
        return Ok(Connection {
            handle,
            recv_endpoint: input,
            send_endpoint: output,
        });
    }
}

/// Walk the device's descriptors and find the correct endpoints.
/// Returns Some((input_endpoint, output_endpoint, interface_number)) on success.
fn find_endpoints(dev: &Device<GlobalContext>) -> Option<(u8, u8, u8)> {
    // Walk in the order configuration descriptor -> interface descriptor -> endpoint descripto
    // Get configuration descriptors
    let conf = dev.active_config_descriptor().unwrap();
    let ifaces = conf.interfaces();

    // Get interface descriptors of the correct class and correct number of endpoints
    let mut iface_descriptor: Option<InterfaceDescriptor> = None;
    let mut iface_num: Option<u8> = None;
    for iface in ifaces {
        for descr in iface.descriptors() {
            if (descr.class_code() == USB_CLASS_CDC_DATA) && (descr.num_endpoints() == 2) {
                iface_num = Some(descr.interface_number());
                iface_descriptor = Some(descr);
                break;
            }
        }
        if iface_descriptor.is_some() {
            break;
        }
    }

    // Of these endpoints, find the correct input and output ones
    let iface_descriptor = iface_descriptor.expect("Failed to find matching interface descriptor");
    let endpoint_descriptors = iface_descriptor.endpoint_descriptors();
    let mut input: Option<u8> = None;
    let mut output: Option<u8> = None;
    for descr in endpoint_descriptors {
        match descr.direction() {
            Direction::In => input = Some(descr.address()),
            Direction::Out => output = Some(descr.address()),
        }
    }
    if input.is_none() {
        panic!("Failed to find input descriptor");
    }
    if output.is_none() {
        panic!("Failed to find output descriptor");
    }

    return Some((input.unwrap(), output.unwrap(), iface_num.unwrap()));
}

impl Communicator for Connection {
    /// Sends the given data to the device.
    /// Blocks until all data could be sent or an error occurs.
    fn send(&mut self, data: &[u8]) -> IOResult<()> {
        self.handle
            .write_bulk(self.send_endpoint, data, Duration::from_secs(10))
            .unwrap();

        log::trace!(target: "USB", "Send: {:?}", data);
        return Ok(());
    }

    fn recv_exact(&mut self, how_much: usize) -> IOResult<Vec<u8>> {
        let mut buf = vec![];
        buf.resize(how_much, 0);
        match self
            .handle
            .read_bulk(self.recv_endpoint, &mut buf, Duration::from_secs(10))
        {
            // FIXME: Should read repeatedly until desired data volume is reached instead
            Ok(read) => buf.resize(read, 0),
            Err(e) => panic!("{}", e),
        }

        log::trace!(target: "USB", "Recv: {:?}", buf);
        return Ok(buf);
    }
}
