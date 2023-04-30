//! This crate implements support for the Odin .tar.md5 file format,
//! in particular parsing it's metadata and performing hash validation.

#![allow(clippy::needless_return)]

mod error;
pub use error::*;
mod metadata;
pub use metadata::*;
#[cfg(test)]
mod integration_tests;

use std::io::{Read, Seek, SeekFrom};

pub use tar;
const MD_5: &str = "218789cf915d52335c8d699169b31b99";

/// An Odin tar archive.
pub struct OdinTar<R: ?Sized + Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> OdinTar<R> {
    /// Create by wrapping the given reader.
    ///
    /// The `Reader` must also implement `Seek` to allow for efficient metadata retrieval.
    pub fn from_reader(r: R) -> OdinTar<R> {
        return OdinTar { reader: r };
    }

    /// Validate the checksum of archive contents based on the Odin metadata in it.
    ///
    /// Quite slow, as the entire archive contents have to be read once.
    pub fn validate(&mut self) -> Result<(), OdinTarError> {
        let expected = self.metadata()?.md5;
        let mut ctx = md5::Context::new();

        // Hash the file, 1MiB at a time.
        let mut buf: Vec<u8> = vec![];
        buf.resize(1024 * 1024, 0);
        // Ignore the appended filename and hash.
        let data_to_discard_pos = self.hashed_offset()?;
        let mut read_total: u64 = 0;

        loop {
            let read: usize = self.reader.read(&mut buf)?;
            if read == 0 {
                // Assume reader is exhausted and all data has been processed
                break;
            }
            let read_u64: u64 = read.try_into()?;
            read_total += read_u64;

            // Ignore any metadata we may have read
            let read_total_signed: i64 = read_total.try_into()?;
            let data_to_discard_pos_signed: i64 = data_to_discard_pos.try_into()?;
            let metadata_overlap: usize =
                std::cmp::max(0, read_total_signed - data_to_discard_pos_signed).try_into()?;
            let buf_used = &buf[0..read - metadata_overlap];

            ctx.consume(buf_used);
        }
        let got = ctx.compute();
        let got: String = format!("{got:x}");
        if got != expected {
            return Err(OdinTarError::ChecksumError(expected, got));
        } else {
            return Ok(());
        }
    }

    /// Returns offset into the underlying `Reader` at which data which should not be hashed begins.
    fn hashed_offset(&mut self) -> Result<u64, OdinTarError> {
        // We simply search for the second occurrence of 0x0A (LF) in the file from the end.
        // Doing this on a reader is a pain, so we read the end of the file into a buffer.
        // NOTE: This assumes metadata no longer than 768 bytes.
        self.reader.seek(SeekFrom::End(-768))?;
        let mut buf: Vec<u8> = vec![];
        buf.resize(768, 0);
        self.reader.read_exact(&mut buf)?;
        let reader_size = self.reader.seek(SeekFrom::End(0))?;
        self.reader.rewind()?;

        let occurrence_indices = buf
            .iter()
            .enumerate()
            .filter(|(_, x)| **x == 0x0A)
            .map(|(i, _)| i + 1)
            .collect::<Vec<usize>>();
        if occurrence_indices.len() < 2 {
            return Err(OdinTarError::MetadataError);
        }

        let offset = occurrence_indices[occurrence_indices.len() - 2] as u64 + reader_size - 768;

        return Ok(offset);
    }

    /// Returns offset into the underlying `Reader` at which metadata begins.
    fn metadata_offset(&mut self) -> Result<u64, OdinTarError> {
        // Assume that looking for the last sequence of zeroes is a sane substitute for actually parsing
        // the tar to find terminating blocks.
        // Assume that this is in the sequence (metadata longer than this probably doesn't exist)
        // TODO: This approach isn't very robust. Probably better to find the start of the sequence and skip it then.
        self.reader.seek(SeekFrom::End(-768))?;
        // Find start of metadata
        loop {
            let mut sample: [u8; 4] = [0, 0, 0, 0];
            self.reader.read_exact(&mut sample)?;
            if sample != [0, 0, 0, 0] {
                // We've gone one too far
                self.reader.seek(SeekFrom::Current(-4))?;
                break;
            }
            self.reader.seek(SeekFrom::Current(4))?;
        }
        // Hack to get the seek position
        let pos = self.reader.seek(SeekFrom::Current(0))?;
        self.reader.rewind()?;
        return Ok(pos);
    }

    /// Parse the Odin-specific metadata out of the archive.
    pub fn metadata(&mut self) -> Result<Metadata, OdinTarError> {
        let pos = self.metadata_offset()?;
        self.reader.seek(SeekFrom::Start(pos))?;

        // Extract and parse it
        let mut data: String = String::with_capacity(512);
        self.reader.read_to_string(&mut data)?;
        let metadata = Metadata::from_file_trailer(&data)?;

        self.reader.rewind()?;

        return Ok(metadata);
    }

    /// Return the underlying tar archive, consuming the instance.
    ///
    /// Use this to get access to the archive's files.
    pub fn archive(self) -> tar::Archive<R> {
        return tar::Archive::new(self.reader);
    }
}
