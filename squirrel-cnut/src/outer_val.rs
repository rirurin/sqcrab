use std::error::Error;
use std::io::{Cursor, Seek, SeekFrom};
use crate::from_slice;
use crate::object::BinObject;
use crate::utils::slice::FromSlice;
use crate::utils::endianness::Endianness;

#[derive(Debug)]
#[allow(dead_code)]
pub struct OuterValue {
    ty: i32,
    src: BinObject,
    name: BinObject
}

impl OuterValue {
    pub fn new<E: Endianness>(stream: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
        let slice = &stream.get_ref()[stream.position() as usize..];
        let ty = from_slice!(slice, i32, E, 0x0);
        stream.seek(SeekFrom::Current(0x4))?;
        let src = BinObject::new::<E>(stream)?;
        let _: &str = (&src).try_into()?;
        let name = BinObject::new::<E>(stream)?;
        let _: &str = (&name).try_into()?;
        Ok(Self { ty, src, name })
    }

    pub fn get_src(&self) -> &str {
        (&self.src).try_into().unwrap()
    }

    pub fn get_name(&self) -> &str {
        (&self.name).try_into().unwrap()
    }
}