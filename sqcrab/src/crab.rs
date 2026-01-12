use squirrel::vm::{SquirrelVM, SquirrelVMBuilder};
use crate::debug::{CrabDebugger, ScriptDebugger};

#[derive(Debug)]
pub struct SqCrab<D: ScriptDebugger> {
    debugger: D,
    sqvm: SquirrelVM
}

impl<D: ScriptDebugger> SqCrab<D> {
    pub fn new() -> SquirrelVMBuilder {
        SquirrelVM::new()
    }
}