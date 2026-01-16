#![allow(unused_imports)]
use std::fmt::{Debug, Formatter, Pointer};
use std::io::{Cursor, Seek, SeekFrom};
use squirrel_sys::bindings::root::*;
use crate::from_slice;
use crate::utils::slice::FromSlice;
use crate::utils::endianness::Endianness;
use crate::utils::error::SquirrelBinaryError;
use crate::utils::reader::try_advance;

// #[derive(Debug)]
pub enum BinObject {
    Integer(i64),
    Float(f32),
    String(*const str),
}

impl BinObject {
    pub fn new<E: Endianness>(stream: &mut Cursor<&[u8]>) -> Result<Self, SquirrelBinaryError> {
        let slice = &stream.get_ref()[stream.position() as usize..];
        let type_id = from_slice!(slice, i32, E, 0x0);
        #[allow(non_upper_case_globals)]
        match type_id {
            tagSQObjectType_OT_STRING => {
                let len = from_slice!(slice, i64, E, 0x4);
                try_advance(stream, 12 + len)?;
                Ok(Self::String(std::str::from_utf8(&slice[12..12 + len as usize])
                    .map_err(|e| SquirrelBinaryError::Utf8Error(e))?))
            },
            tagSQObjectType_OT_INTEGER => {
                try_advance(stream, 8)?;
                Ok(Self::Integer(from_slice!(slice, i32, E, 0x4) as i64))
            },
            tagSQObjectType_OT_FLOAT => {
                try_advance(stream, 8)?;
                Ok(Self::Float(from_slice!(slice, f32, E, 0x4)))
            },
            _ => Err(SquirrelBinaryError::UnimplementedBinObject(type_id))
        }
    }

    pub fn type_id(&self) -> i32 {
        match self {
            Self::Integer(_) => tagSQObjectType_OT_INTEGER,
            Self::Float(_) => tagSQObjectType_OT_FLOAT,
            Self::String(_) => tagSQObjectType_OT_STRING,
        }
    }
}

impl Debug for BinObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(i) => write!(f, "BinObject::Integer({})", *i),
            Self::Float(i) => write!(f, "BinObject::Float({})", *i),
            Self::String(i) => write!(f, "BinObject::String('{}')", unsafe { &**i }),
        }
    }
}

impl TryFrom<&BinObject> for i64 {
    type Error = SquirrelBinaryError;

    fn try_from(value: &BinObject) -> Result<Self, Self::Error> {
        match value {
            BinObject::Integer(i) => Ok(*i),
            _ => Err(SquirrelBinaryError::WrongBinObjectType(tagSQObjectType_OT_INTEGER))
        }
    }
}

impl TryFrom<&BinObject> for f32 {
    type Error = SquirrelBinaryError;

    fn try_from(value: &BinObject) -> Result<Self, Self::Error> {
        match value {
            BinObject::Float(i) => Ok(*i),
            _ => Err(SquirrelBinaryError::WrongBinObjectType(tagSQObjectType_OT_FLOAT))
        }
    }
}

impl TryFrom<&BinObject> for &str {
    type Error = SquirrelBinaryError;

    fn try_from(value: &BinObject) -> Result<Self, Self::Error> {
        match value {
            BinObject::String(i) => Ok(unsafe { &**i }),
            _ => Err(SquirrelBinaryError::WrongBinObjectType(tagSQObjectType_OT_STRING))
        }
    }
}