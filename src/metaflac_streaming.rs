use crate::error::Error;
use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub fn read_from(path: PathBuf, data: &mut Vec<u8>) -> Result<VorbisComment, Error> {
    let mut reader = File::open(&*path)?;
    let mut ident = [0; 4];
    reader.read_exact(&mut ident)?;

    if &ident != b"fLaC" {
        return Err(Error::InvalidFlacHeader(path.into()));
    }

    read_tags(&mut reader, path, data)
}

// See documentation: https://xiph.org/flac/format.html
fn read_tags(reader: &mut File, path: PathBuf, data: &mut Vec<u8>) -> Result<VorbisComment, Error> {
    loop {
        let mut buf = [0; 4];

        reader.read_exact(&mut buf)?;
        let is_last = (buf[0] & 0b1000_0000) != 0;
        let blocktype_byte = buf[0] & 0b0111_1111;
        let length = u32::from_be_bytes(buf) & 0x00FFFFFF;

        if blocktype_byte == 4 {
            data.clear();
            reader.take(length as u64).read_to_end(data)?;
            return Ok(VorbisComment::from_bytes(path, data));
        } else if is_last {
            return Ok(VorbisComment::empty(path));
        } else {
            reader.seek(SeekFrom::Current(length as i64))?;
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct VorbisComment {
    pub path: PathBuf,
    num_comments: u32,
    i: usize,
    curr: u32,
}

impl VorbisComment {
    pub fn empty(path: PathBuf) -> VorbisComment {
        VorbisComment {
            path,
            num_comments: 0,
            i: 0,
            curr: 0,
        }
    }

    pub fn from_bytes(path: PathBuf, bytes: &Vec<u8>) -> VorbisComment {
        let vendor_length = u32::from_le_bytes((&bytes[0..4]).try_into().unwrap()) as usize;
        let num_comments = u32::from_le_bytes(
            (&bytes[4 + vendor_length..8 + vendor_length])
                .try_into()
                .unwrap(),
        );
        VorbisComment {
            path,
            num_comments,
            i: 8 + vendor_length,
            curr: 0,
        }
    }

    pub fn cur<'a, 'b>(
        &'a self,
        bytes: &'b Vec<u8>,
    ) -> Result<Option<(&'a Path, &'b str, &'b str)>, Error> {
        if self.curr <= self.num_comments {
            let comment_length =
                u32::from_le_bytes((bytes[self.i..self.i + 4]).try_into().unwrap()) as usize;

            let (key, value) =
                read_vorbis_comment(&bytes[self.i + 4..self.i + 4 + comment_length])?;

            Ok(Some((&*self.path, key, value)))
        } else {
            Ok(None)
        }
    }

    pub fn next(&mut self, bytes: &Vec<u8>) -> bool {
        if self.curr == 0 && self.num_comments != 0 {
            self.curr = 1;
            true
        } else if self.curr < self.num_comments {
            self.curr += 1;
            self.i +=
                4 + u32::from_le_bytes((&bytes[self.i..self.i + 4]).try_into().unwrap()) as usize;
            true
        } else {
            self.curr += 1;
            false
        }
    }
}

fn read_vorbis_comment(bytes: &[u8]) -> Result<(&str, &str), Error> {
    let comments = std::str::from_utf8(bytes)?;

    let mut comments_split = comments.split('=');
    let key = comments_split
        .next()
        .ok_or_else(|| Error::MalformedVorbisComment(comments.into()))?;
    let value = comments_split
        .next()
        .ok_or_else(|| Error::MalformedVorbisComment(comments.into()))?;
    Ok((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_all_tags() {
        let path = PathBuf::from("test-data/test-tag.flac");
        let mut vorbis_comments = read_from(path.clone(), Vec::new()).unwrap();

        assert!(vorbis_comments.next());

        assert_eq!(
            (&*path, "TEST", "1"),
            vorbis_comments.cur().unwrap().unwrap()
        );

        assert!(vorbis_comments.next());

        assert_eq!(
            (&*path, "TEST", "2"),
            vorbis_comments.cur().unwrap().unwrap()
        );

        assert!(!vorbis_comments.next());
    }
}
