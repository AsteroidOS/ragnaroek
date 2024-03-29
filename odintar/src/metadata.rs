use crate::OdinTarError;

/// Odin-specific metadata appended to the archive contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// Identifier for the firmware build contained within.
    pub build_id: u64,
    /// Original size of files without compression or metadata trailer.
    pub orig_size: u64,
    /// MD5 checksum of archive without metadata trailer.
    pub md5: String,
    /// Filename used to identify contents before packing.
    pub orig_file_name: String,
}

impl Metadata {
    /// Parse the metadata from the given Odin .tar.md5 file trailer
    /// (the part after the 2 empty blocks terminating the archive).
    pub fn from_file_trailer(data: &str) -> Result<Metadata, OdinTarError> {
        let mut build_id: Option<u64> = None;
        let mut orig_size: Option<u64> = None;
        for line in data.lines() {
            if line.contains("BUILD_ID") {
                let id = line.rsplit_once(':').unwrap().1;
                let id: u64 = str::parse(id)?;
                build_id = Some(id);
            } else if line.contains("original_tar_file_size") {
                let size = line.rsplit_once(':').unwrap().1;
                let size: u64 = str::parse(size)?;
                orig_size = Some(size);
            }
        }
        // This entry does not follow the key:value format of the rest, so assume it can only sanely ever be last
        // There's (sometimes?) a newline after the last entry, trim it so we don't confuse rsplit
        let last_line = data.trim_end_matches('\n').rsplit_once('\n').unwrap().1;
        let (hash, name) = last_line.split_once("  ").unwrap();
        let md5 = String::from(hash);
        let orig_file_name = String::from(name);

        return Ok(Metadata {
            build_id: build_id.unwrap(),
            orig_size: orig_size.unwrap(),
            md5,
            orig_file_name,
        });
    }
}

#[test]
fn test_metadata_parse() {
    // Taken from a Galaxy A40 BL file
    let input: &str = "Show the build information\nRBS BUILD_ID:58944467\noriginal_tar_file_size:3368960\n218789cf915d52335c8d699169b31b99  BL_A405FNXXU4CVK1_CL25488227_QB58944467_REV00_user_low_ship.tar\n";
    let expected = Metadata {
        build_id: 58944467,
        orig_size: 3368960,
        md5: String::from("218789cf915d52335c8d699169b31b99"),
        orig_file_name: String::from(
            "BL_A405FNXXU4CVK1_CL25488227_QB58944467_REV00_user_low_ship.tar",
        ),
    };
    let got = Metadata::from_file_trailer(input).unwrap();
    assert_eq!(expected, got);
}
