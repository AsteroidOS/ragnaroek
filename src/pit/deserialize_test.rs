use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use crate::Error;

use super::Pit;

const PIT_PATH: &str = "src/pit/testdata/";

#[test]
fn test_deserialize() {
    // Enumerate all PIT files we have in the test directory
    let pit_dir = Path::new(PIT_PATH);
    let mut failed: Vec<(String, Error)> = Vec::new();

    for entry in fs::read_dir(pit_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let path = Path::new(&path);
        let mut f = File::open(path).unwrap();
        let mut data: Vec<u8> = Vec::new();
        f.read_to_end(&mut data).unwrap();

        let pit = Pit::deserialize(&data);

        // TODO: More thorough testing of the shape of the data

        if !pit.is_ok() {
            failed.push((path.to_str().unwrap().to_string(), pit.err().unwrap()));
        }
    }

    if !failed.is_empty() {
        panic!("Failed to parse some PIT files: {:?}", failed);
    }
}
