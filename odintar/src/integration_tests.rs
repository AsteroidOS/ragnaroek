use std::fs::File;
use std::io::Cursor;

use crate::*;

const TEST_FILE: &str =
    "./testdata/BL_A405FNXXU4CVK1_CL25488227_QB58944467_REV00_user_low_ship.tar.md5";

#[test]
fn test_read_metadata() {
    let expected = Metadata {
        build_id: 58944467,
        orig_size: 3368960,
        md5: String::from("218789cf915d52335c8d699169b31b99"),
        orig_file_name: String::from(
            "BL_A405FNXXU4CVK1_CL25488227_QB58944467_REV00_user_low_ship.tar",
        ),
    };

    let f = File::open(TEST_FILE).unwrap();
    let mut archive = OdinTar::from_reader(f);

    let metadata = archive.metadata();

    assert!(metadata.is_ok());
    assert_eq!(expected, metadata.unwrap());
}

#[test]
fn test_validate_ok() {
    let f = File::open(TEST_FILE).unwrap();
    let mut archive = OdinTar::from_reader(f);

    let v = archive.validate();
    panic!("{v:?}");
}

#[test]
fn test_validate_corrupted() {
    let mut f = File::open(TEST_FILE).unwrap();
    // Simulate corruption by flipping a byte
    let mut buf: Vec<u8> = vec![];
    f.read_to_end(&mut buf).unwrap();
    buf[256] = !buf[256];

    let mut archive = OdinTar::from_reader(Cursor::new(buf));
    let v = archive.validate();

    assert!(v.is_err());
    match v.as_ref().err().unwrap() {
        OdinTarError::ChecksumError(_, _) => {}
        _ => panic!("Wrong error! Expected ChecksumError, got {v:?}"),
    }
}

#[test]
fn test_read_contents() {
    let expected: [&str; 4] = [
        "cm.bin.lz4",
        "param.bin.lz4",
        "sboot.bin.lz4",
        "vbmeta.img.lz4",
    ];

    // We don't test whether the contents are correct (that's the job of the tar crate),
    // only whether we proxied the iterator through correctly.
    let f = File::open(TEST_FILE).unwrap();
    let archive = OdinTar::from_reader(f);

    let mut count = 0;
    for f in archive.archive().entries().unwrap() {
        let temp1 = f.unwrap();
        let temp2 = temp1.path().unwrap();
        let name = temp2.as_os_str().to_str().unwrap();
        assert!(expected.contains(&name));
        count += 1;
    }
    assert_eq!(expected.len(), count);
}
