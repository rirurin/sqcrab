use std::error::Error;
use std::io::{Cursor, Seek, SeekFrom};
use crate::from_slice;
use crate::object::BinObject;
use crate::utils::slice::FromSlice;
use crate::utils::endianness::Endianness;

#[derive(Debug)]
#[allow(dead_code)]
pub struct LocalVar {
    name: BinObject,
    pos: i64,
    start_op: i64,
    end_op: i64
}

impl LocalVar {
    pub fn new<E: Endianness>(stream: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
        let name = BinObject::new::<E>(stream)?;
        let _: &str = (&name).try_into()?;
        let slice = &stream.get_ref()[stream.position() as usize..];
        let pos = from_slice!(slice, i64, E, 0x0);
        let start_op = from_slice!(slice, i64, E, 0x8);
        let end_op = from_slice!(slice, i64, E, 0x10);
        stream.seek(SeekFrom::Current(0x18))?;
        Ok(Self { name, pos, start_op, end_op })
    }
}