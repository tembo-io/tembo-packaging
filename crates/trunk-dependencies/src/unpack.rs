use anyhow::Result;
use std::{
    ffi::OsStr,
    io::{Cursor, Read},
    path::PathBuf,
};
use tar::EntryType;

use flate2::read::GzDecoder;

pub struct Archive {
    pub shared_objects: Vec<Entry>,
}

pub struct Entry {
    #[expect(unused)]
    pub path: PathBuf,
    pub contents: Vec<u8>,
}

// Decompress this tar.gz in-memory, keeping only shared object files
pub fn decompress_in_memory(tar_gz: &[u8]) -> Result<Archive> {
    let mut buf = Vec::with_capacity(tar_gz.len() * 8);
    GzDecoder::new(tar_gz).read_to_end(&mut buf)?;

    let mut archive = tar::Archive::new(Cursor::new(buf));

    let mut entries = Vec::new();

    for maybe_entry in archive.entries()? {
        let mut entry = maybe_entry?;
        let header = entry.header();
        let entry_size = header.entry_size().unwrap_or(12500);

        match header.entry_type() {
            EntryType::Regular => {}
            other => {
                eprintln!(
                    "decompressing: Found a {:?} file, expected Regular. Ignoring",
                    other
                );
                continue;
            }
        }

        let path = entry.path()?.to_path_buf();

        if path.extension() != Some(OsStr::new("so")) {
            continue;
        }

        let contents = {
            let mut buf = Vec::with_capacity(entry_size as usize);

            entry.read_to_end(&mut buf)?;
            buf
        };

        entries.push(Entry { path, contents });
    }

    Ok(Archive {
        shared_objects: entries,
    })
}
