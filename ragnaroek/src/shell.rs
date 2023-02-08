//! This module implements the backend for an interactive command shell exposed by some versions of LOKE.
//! See https://samsung-loki.github.io/samsung-docs/docs/Odin/Commands/ for details.

use std::{thread, time::Duration};

use crate::{error::*, Communicator};

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(2);

/// Sends a command to the bootloader's interactive shell.
/// Returns target's response if it was received, otherwise `Ok(None)`.
pub fn exchange_cmd(c: &mut Box<dyn Communicator>, cmd: &str) -> Result<Option<String>> {
    let cmd = format!("PROMPT {cmd}");
    log::info!(target: "SHELL", "Command: {}", cmd);
    c.send(cmd.as_bytes())?;

    // Commands don't have a fixed format, so simply try reading until we haven't gotten new data for a certain time.
    let mut data: Vec<u8> = Vec::new();
    loop {
        log::trace!(target: "SHELL", "Loop");
        let old_data_size: usize = data.len();
        data.extend_from_slice(&c.recv()?);
        log::trace!(target: "SHELL", "Current response buffer: {:?}", data);
        if data.len() == old_data_size {
            // Assume that target is done
            if data.is_empty() {
                return Ok(None);
            }
            return Ok(Some(
                String::from_utf8(data).expect("Target sent invalid UTF-8"),
            ));
        }

        // Wait for next read
        log::trace!(target: "SHELL", "Sleep");
        thread::sleep(RESPONSE_TIMEOUT);
    }
}
