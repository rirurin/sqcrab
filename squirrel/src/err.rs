use std::fmt::{Debug, Display, Formatter};
use std::error::Error;
use std::str::Utf8Error;
use squirrel_sys::bindings::root::{SQInteger, HSQUIRRELVM};

#[derive(Debug)]
pub enum SquirrelError {
    CouldNotCompileSource,
    GetWrongObjectType,
    ErrorWhileCalling,
    CouldNotReadBytecode,
    CouldNotSuspendVM,
    CouldNotWakeupVM,
    Utf8Error(Utf8Error),
    CouldNotAddFunction,
    CouldNotSetNativeClosureName,
    ObjectTypeDoesNotMatch,
    CouldNotFindFunction(String),
    ForeignPointerNotSet
}

impl Error for SquirrelError {}
impl Display for SquirrelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        <Self as Debug>::fmt(self, f)
    }
}

pub(crate) type ErrorCallback = unsafe extern "C" fn(HSQUIRRELVM) -> SQInteger;