use std::fmt::{Debug, Display, Formatter};
use std::error::Error;

#[derive(Debug)]
pub enum SquirrelError {
    CouldNotCompileBuffer,
    GetWrongObjectType
}

impl Error for SquirrelError {}
impl Display for SquirrelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        <Self as Debug>::fmt(self, f)
    }
}