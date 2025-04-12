use flate2::read::GzDecoder;
use std::{
    io::{self, Read},
    path::Path,
};
use tar::Archive;

use crate::digest::{Digest, read_digests};

pub enum EntryType {
    Regular(Box<Path>),
    Symlink {
        destination: Box<Path>,
        symlink_target: Box<Path>,
    },
}

pub struct UnpackedFile {
    pub path: Box<Path>,
    pub contents: Vec<u8>,
    pub entry_type: tar::EntryType,
}

pub fn unpack_files(body: ureq::Body) -> Result<(Vec<UnpackedFile>, Digest), io::Error> {
    let tar = GzDecoder::new(body.into_reader());
    let mut archive = Archive::new(tar);
    let entries = archive.entries()?;
    let mut files = Vec::new();
    let mut digests = None;
    for entry in entries {
        let mut entry = entry?;
        let header = entry.header();
        let entry_type = header.entry_type();

        let path: Box<Path> = entry.path()?.into();

        let mut contents = Vec::new();
        entry.read_to_end(&mut contents)?;

        if &*path == Path::new("digests") {
            digests = Some(read_digests(contents)?);
        } else {
            files.push(UnpackedFile {
                path,
                contents,
                entry_type,
            });
        }
    }
    let digests = digests
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No digests file found"))?;

    Ok((files, digests))
}
