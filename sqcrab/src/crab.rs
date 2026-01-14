use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
use squirrel::err::SquirrelError;
use squirrel::vm::{SquirrelVM, SquirrelVMBuilder, ThreadSafeSquirrelVMPointer};
use crate::debug::{CrabDebugger, DebuggerFlags, ScriptDebugger};
use crate::domain::DomainRegistrar;

#[derive(Debug)]
pub struct SqCrabBuilder {
    debug_flags: DebuggerFlags,
    inner: SquirrelVMBuilder
}

impl Deref for SqCrabBuilder {
    type Target = SquirrelVMBuilder;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SqCrabBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl SqCrabBuilder {
    fn new(debug_flags: DebuggerFlags) -> Self {
        Self { debug_flags, inner: SquirrelVM::new() }
    }

    pub fn build(self) -> SqCrab<CrabDebugger> {
        let mut vm = Box::new(self.inner.build());
        if unsafe { !squirrel::vm::check_squirrel_handle(vm.as_ref()) } {
            unsafe { squirrel::vm::add_squirrel_handle(vm.as_mut()) };
        }
        SqCrab::<CrabDebugger>::from_parts(vm, CrabDebugger::new(DebuggerFlags::default()))
    }
}

// type CrabDebuggerMap = HashMap<ThreadSafeSquirrelVMPointer, >;

// pub static CRAB_DEBUGGER_INSTANCES: Mutex<Option<HashMap<>>>

#[derive(Debug)]
pub struct SqCrab<D: ScriptDebugger> {
    debugger: D,
    sqvm: Box<SquirrelVM>
}

impl<D: ScriptDebugger> Drop for SqCrab<D> {
    fn drop(&mut self) {
    }
}

impl SqCrab<CrabDebugger> {
    pub fn new() -> SqCrabBuilder {
        SqCrabBuilder::new(DebuggerFlags::default())
    }
}

impl<D: ScriptDebugger> SqCrab<D> {
    pub fn from_parts(sqvm: Box<SquirrelVM>, debugger: D) -> Self {
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
        self.sqvm.as_ref()
    }
}

impl<D: ScriptDebugger> DerefMut for SqCrab<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.sqvm.as_mut()
    }
}