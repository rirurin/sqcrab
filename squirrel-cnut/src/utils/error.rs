use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::Utf8Error;
use squirrel_sys::bindings::root::tagSQObjectType;

#[derive(Debug)]
pub enum SquirrelBinaryError {
    InvalidFAFAHeader,
    InvalidSQIRError,
    InvalidTail,
    InvalidOpcode(u8),
    UnimplementedBinObject(tagSQObjectType),
    WrongBinObjectType(tagSQObjectType),
    OutOfRange,
    ExpectedPart,
    InvalidBitwiseOp(u8),
    InvalidCmpOp(u8),
    InvalidNewObjectType(u8),
    InvalidAppendArrayType(u8),

    Utf8Error(Utf8Error),
    IoError(std::io::Error)
}

impl Display for SquirrelBinaryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl Error for SquirrelBinaryError {}