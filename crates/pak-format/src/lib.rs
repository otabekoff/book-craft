//! Where a book's `book.pak` lives inside a portable reader `.exe`.
//!
//! The pak (a zip archive) is appended to the executable, followed by a
//! 24-byte trailer: `BCPAK001`, the pak's offset and its length (both u64 LE).
//! Everything is padded to 8 bytes so Authenticode signing, which appends its
//! certificate table after the file's end, keeps the trailer intact and findable.
//!
//! Installed copies keep `book.pak` as a separate file next to the exe instead.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub const MAGIC: &[u8; 8] = b"BCPAK001";
pub const TRAILER_LEN: u64 = 24;
pub const PAK_FILE: &str = "book.pak";

/// Appends the pak file and the trailer to the executable at `exe`.
pub fn append_file(exe: &Path, pak: &Path) -> io::Result<()> {
    let mut src = File::open(pak)?;
    write_payload(exe, &mut src)
}

/// Appends `pak` bytes and the trailer to the executable at `exe`.
pub fn append(exe: &Path, pak: &[u8]) -> io::Result<()> {
    write_payload(exe, &mut io::Cursor::new(pak))
}

fn write_payload(exe: &Path, src: &mut dyn Read) -> io::Result<()> {
    let mut f = std::fs::OpenOptions::new().append(true).open(exe)?;
    let len = f.metadata()?.len();
    let pad_before = (8 - len % 8) % 8;
    f.write_all(&vec![0u8; pad_before as usize])?;
    let offset = len + pad_before;
    let written = io::copy(src, &mut f)?;
    let after = offset + written;
    let pad_after = (8 - after % 8) % 8;
    f.write_all(&vec![0u8; pad_after as usize])?;
    f.write_all(MAGIC)?;
    f.write_all(&offset.to_le_bytes())?;
    f.write_all(&written.to_le_bytes())?;
    f.flush()
}

/// Finds an embedded pak. Returns `(offset, length)`.
pub fn find(file: &mut File) -> io::Result<Option<(u64, u64)>> {
    let size = file.metadata()?.len();
    // A signed file ends with its certificate table; the trailer sits just before it.
    let end = match certificate_table_start(file)? {
        Some(start) if start <= size => start,
        _ => size,
    };
    // Allow a little zero padding between our trailer and the end.
    let mut probe = end;
    for _ in 0..4 {
        if probe < TRAILER_LEN {
            break;
        }
        let mut buf = [0u8; 24];
        file.seek(SeekFrom::Start(probe - TRAILER_LEN))?;
        file.read_exact(&mut buf)?;
        if &buf[..8] == MAGIC {
            let offset = u64::from_le_bytes(buf[8..16].try_into().unwrap());
            let len = u64::from_le_bytes(buf[16..24].try_into().unwrap());
            if offset.checked_add(len).is_some_and(|e| e <= probe - TRAILER_LEN) {
                return Ok(Some((offset, len)));
            }
            return Ok(None);
        }
        if buf[16..24] != [0; 8] {
            break;
        }
        probe -= 8;
    }
    Ok(None)
}

/// Reads the PE security data directory (the Authenticode certificate table).
fn certificate_table_start(file: &mut File) -> io::Result<Option<u64>> {
    let mut mz = [0u8; 64];
    file.seek(SeekFrom::Start(0))?;
    if file.read(&mut mz)? < 64 || &mz[..2] != b"MZ" {
        return Ok(None);
    }
    let pe = u32::from_le_bytes(mz[60..64].try_into().unwrap()) as u64;
    let mut head = [0u8; 24 + 2];
    file.seek(SeekFrom::Start(pe))?;
    file.read_exact(&mut head)?;
    if &head[..4] != b"PE\0\0" {
        return Ok(None);
    }
    let magic = u16::from_le_bytes([head[24], head[25]]);
    // Offset of the data directories inside the optional header.
    let dirs = match magic {
        0x10b => 96,
        0x20b => 112,
        _ => return Ok(None),
    };
    // Security is data directory #4 (8 bytes each).
    let at = pe + 24 + dirs + 4 * 8;
    let mut entry = [0u8; 8];
    file.seek(SeekFrom::Start(at))?;
    file.read_exact(&mut entry)?;
    let addr = u32::from_le_bytes(entry[..4].try_into().unwrap()) as u64;
    let size = u32::from_le_bytes(entry[4..].try_into().unwrap()) as u64;
    Ok(if addr != 0 && size != 0 { Some(addr) } else { None })
}

/// A `Read + Seek` view of a byte range inside a file (for the zip reader).
pub struct Slice {
    file: File,
    start: u64,
    len: u64,
    pos: u64,
}

impl Slice {
    pub fn new(mut file: File, start: u64, len: u64) -> io::Result<Self> {
        file.seek(SeekFrom::Start(start))?;
        Ok(Self { file, start, len, pos: 0 })
    }
}

impl Read for Slice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let left = self.len.saturating_sub(self.pos);
        if left == 0 {
            return Ok(0);
        }
        let n = (buf.len() as u64).min(left) as usize;
        let read = self.file.read(&mut buf[..n])?;
        self.pos += read as u64;
        Ok(read)
    }
}

impl Seek for Slice {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let target = match pos {
            SeekFrom::Start(p) => p as i128,
            SeekFrom::End(d) => self.len as i128 + d as i128,
            SeekFrom::Current(d) => self.pos as i128 + d as i128,
        };
        if target < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "seek before start"));
        }
        self.pos = target as u64;
        self.file.seek(SeekFrom::Start(self.start + self.pos))?;
        Ok(self.pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_with_padding_and_fake_signature() {
        let dir = std::env::temp_dir().join(format!("pakfmt-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("a.exe");
        std::fs::write(&exe, b"not really a PE file, 37 bytes long..").unwrap();
        append(&exe, b"PK-zip-bytes").unwrap();
        let len = std::fs::metadata(&exe).unwrap().len();
        assert_eq!(len % 8, 0);
        let mut f = File::open(&exe).unwrap();
        let (off, n) = find(&mut f).unwrap().unwrap();
        assert_eq!(n, 12);
        let mut s = Slice::new(File::open(&exe).unwrap(), off, n).unwrap();
        let mut v = Vec::new();
        s.read_to_end(&mut v).unwrap();
        assert_eq!(v, b"PK-zip-bytes");

        // Extra zero padding after the trailer is tolerated.
        std::fs::OpenOptions::new().append(true).open(&exe).unwrap().write_all(&[0; 8]).unwrap();
        let mut f = File::open(&exe).unwrap();
        assert!(find(&mut f).unwrap().is_some());
        std::fs::remove_dir_all(dir).ok();
    }
}
