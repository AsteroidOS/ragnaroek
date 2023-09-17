#![no_main]

use libfuzzer_sys::fuzz_target;
use pit;

fuzz_target!(|data: &[u8]| {
    let _ = pit::Pit::deserialize(data);
});
