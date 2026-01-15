use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::Utf8Error;

#[derive(Debug)]
pub enum SquirrelBinaryError {
    InvalidFAFAHeader,
    InvalidSQIRError,
    InvalidTail,
    InvalidOpcode(u8),
    UnimplementedBinObject(i32),
    WrongBinObjectType(i32),
    OutOfRange,
    ExpectedTrap,
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