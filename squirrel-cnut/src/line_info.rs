use std::error::Error;
use std::io::{Cursor, Read, Seek, SeekFrom};
use crate::from_slice;
use crate::utils::slice::FromSlice;
use crate::utils::endianness::Endianness;
use crate::utils::reader::bytes_from_stream;

#[repr(C)]
#[derive(Debug)]
pub struct LineInfo {
    line: i64,
    op: i64
}

impl LineInfo {
    pub fn from_generic<E: Endianness, R: Read>(stream: &mut R) -> Result<Self, Box<dyn Error>> {
        let slice = bytes_from_stream::<R, { size_of::<Self>() }>(stream)?;
        Ok(unsafe { std::mem::transmute(slice) })
    }

    pub fn new<E: Endianness>(stream: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
        let slice = &stream.get_ref()[stream.position() as usize..];
        let line = from_slice!(slice, i64, E, 0x0);
        let op = from_slice!(slice, i64, E, 0x8);
        stream.seek(SeekFrom::Current(0x10))?;
        Ok(Self { line, op })
    }
}