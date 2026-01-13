use std::ops::{Deref, DerefMut};
use squirrel::err::SquirrelError;
use squirrel::vm::{SquirrelVM, SquirrelVMBuilder};
use crate::debug::{CrabDebugger, ScriptDebugger};
use crate::domain::DomainRegistrar;

#[derive(Debug)]
pub struct SqCrab<D: ScriptDebugger> {
    debugger: D,
    sqvm: SquirrelVM
}

impl<D: ScriptDebugger> SqCrab<D> {
    pub fn from_parts(sqvm: SquirrelVM, debugger: D) -> Self {
        Self { debugger, sqvm }
    }
    pub fn debugger(&self) -> &D {
        &self.debugger
    }
    pub fn debugger_mut(&mut self) -> &mut D {
        &mut self.debugger
    }
    pub fn register<F: DomainRegistrar>(&mut self) -> Result<(), SquirrelError> {
        F::add_functions(self)
    }
}

impl<D: ScriptDebugger> Deref for SqCrab<D> {
    type Target = SquirrelVM;

    fn deref(&self) -> &Self::Target {
        &self.sqvm
    }
}

impl<D: ScriptDebugger> DerefMut for SqCrab<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sqvm
    }
}