use std::{collections::HashMap, io, ops::Deref, path::Path};

pub struct Digest(pub HashMap<Box<Path>, [u8; 64]>);

impl Deref for Digest {
    type Target = HashMap<Box<Path>, [u8; 64]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn read_digests(contents: Vec<u8>) -> Result<Digest, io::Error> {
    let mut digest_map = HashMap::new();

    let contents = std::str::from_utf8(&contents)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    for line in contents.lines() {
        if line.is_empty() {
            continue;
        }
        println!("'{line}'");
        let rest = line
            .strip_prefix("SHA512 (")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing SHA512 prefix"))?;

        let (file_path, rest) = rest.split_once(")").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Missing enclosing parentheses")
        })?;

        let (_, rest) = rest
            .split_once(" = ")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing = separator"))?;

        let checksum = rest;
        let bytes = decode_hex(checksum)?;

        digest_map.insert(Path::new(file_path).into(), bytes);
    }

    Ok(Digest(digest_map))
}

fn decode_hex(hex_str: &str) -> Result<[u8; 64], io::Error> {
    let mut buf = [0; 64];
    hex::decode_to_slice(hex_str, &mut buf)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use crate::digest::read_digests;

    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn test_read_digests_valid_multiple() {
        let input = br#"
SHA512 (./file1.txt) = cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e
SHA512 (./file2.txt) = 1f40fc92da241694750979ee6cf582f2d5d7d28e18335de05abc54d0560e0f53d5a9d2088a3b41cb2049fdb610ff1f6a08bf27a4f56ff1b1a3abf79c4e6e7e3e
SHA512 (./subdir/file3.bin) = 2c74fd17edafd80e8447b0d46741ee243b7b8a0c7a1d66d0b247f2d8eeaad599c3b45c97e50c276b254100d8a51a93b5fd893e876ba2a7c8c8b826e87eabe950
"#;

        let digests = read_digests(input.to_vec()).expect("Should parse valid digest content");

        assert_eq!(
            digests.get(Path::new("./file1.txt")).unwrap(),
            &decode_hex(
                "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
            ).unwrap()
        );
        assert_eq!(
            digests.get(Path::new("./file2.txt")).unwrap(),
            &decode_hex(
                "1f40fc92da241694750979ee6cf582f2d5d7d28e18335de05abc54d0560e0f53d5a9d2088a3b41cb2049fdb610ff1f6a08bf27a4f56ff1b1a3abf79c4e6e7e3e"
            ).unwrap()
        );
        assert_eq!(
            digests.get(Path::new("./subdir/file3.bin")).unwrap(),
            &decode_hex(
                "2c74fd17edafd80e8447b0d46741ee243b7b8a0c7a1d66d0b247f2d8eeaad599c3b45c97e50c276b254100d8a51a93b5fd893e876ba2a7c8c8b826e87eabe950"
            ).unwrap()
        );

        assert_eq!(digests.len(), 3);
    }

    #[test]
    fn test_read_digests_invalid_prefix() {
        let input = b"NOTSHA512 (./file.txt) = abcdef".to_vec();
        let result = read_digests(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind(), ErrorKind::InvalidData);
        }
    }

    #[test]
    fn test_read_digests_invalid_missing_closing_paren() {
        let input = b"SHA512 (./file.txt = abcdef".to_vec();
        let result = read_digests(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind(), ErrorKind::InvalidData);
        }
    }

    #[test]
    fn test_read_digests_invalid_missing_separator() {
        let input = b"SHA512 (./file.txt) abcdef".to_vec();
        let result = read_digests(input);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind(), ErrorKind::InvalidData);
        }
    }

    #[test]
    fn test_read_digests_ignores_empty_lines() {
        let input = br#"
SHA512 (./file1.txt) = cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e

SHA512 (./file2.txt) = 1f40fc92da241694750979ee6cf582f2d5d7d28e18335de05abc54d0560e0f53d5a9d2088a3b41cb2049fdb610ff1f6a08bf27a4f56ff1b1a3abf79c4e6e7e3e
"#;
        let digests = read_digests(input.to_vec()).expect("Should parse valid digest content");
        assert_eq!(digests.len(), 2);
        assert_eq!(
            digests.get(Path::new("./file1.txt")).unwrap(),
            &decode_hex(
                "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
            ).unwrap()
        );
        assert_eq!(
            digests.get(Path::new("./file2.txt")).unwrap(),
            &decode_hex(
                "1f40fc92da241694750979ee6cf582f2d5d7d28e18335de05abc54d0560e0f53d5a9d2088a3b41cb2049fdb610ff1f6a08bf27a4f56ff1b1a3abf79c4e6e7e3e"
            ).unwrap()
        );
    }
}
