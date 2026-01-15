use std::collections::HashMap;
use std::error::Error;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
use squirrel::err::SquirrelError;
use squirrel::obj_type::UserPointer;
use squirrel::type_cnv::CanSquirrel;
use squirrel::vm::{SquirrelVM, SquirrelVMBuilder, ThreadSafeSquirrelVMPointer};
use crate::debug::{CrabDebugger, DebuggerFlags, ScriptDebugger};
use crate::domain::DomainRegistrar;

#[derive(Debug)]
pub struct SqCrabBuilder<'a, T>
where &'a mut T: CanSquirrel<Into = UserPointer<&'a mut T>>
{
    debug_flags: DebuggerFlags,
    inner: SquirrelVMBuilder,
    _this: PhantomData<&'a T>
}

impl<'a, T> Deref for SqCrabBuilder<'a, T>
where &'a mut T: CanSquirrel<Into = UserPointer<&'a mut T>>
{
    type Target = SquirrelVMBuilder;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> DerefMut for SqCrabBuilder<'a, T>
where &'a mut T: CanSquirrel<Into = UserPointer<&'a mut T>>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, T> SqCrabBuilder<'a, T>
where &'a mut T: CanSquirrel<Into = UserPointer<&'a mut T>>
{
    fn new(debug_flags: DebuggerFlags) -> Self {
        Self { debug_flags, inner: SquirrelVM::new(), _this: PhantomData::<&'a T> }
    }

    pub fn build(self) -> SqCrab<'a, CrabDebugger, T> {
        let mut vm = Box::new(self.inner.build());
        if unsafe { !squirrel::vm::check_squirrel_handle(vm.as_ref()) } {
            unsafe { squirrel::vm::add_squirrel_handle(vm.as_mut()) };
        }
        SqCrab::<'a, CrabDebugger, T>::from_parts(vm, CrabDebugger::new(DebuggerFlags::default()))
    }

    pub fn build2(self) -> SqCrab<'a, CrabDebugger, T> {
        let mut vm = Box::new(self.inner.build());
        if unsafe { !squirrel::vm::check_squirrel_handle(vm.as_ref()) } {
            unsafe { squirrel::vm::add_squirrel_handle(vm.as_mut()) };
        }
        SqCrab::<'a, CrabDebugger, T>::from_parts(vm, CrabDebugger::new(DebuggerFlags::default()))
    }
}

// type CrabDebuggerMap = HashMap<ThreadSafeSquirrelVMPointer, >;

// pub static CRAB_DEBUGGER_INSTANCES: Mutex<Option<HashMap<>>>

#[derive(Debug)]
pub struct SqCrab<'a, D, T>
where D: ScriptDebugger,
      &'a mut T: CanSquirrel<Into = UserPointer<&'a mut T>>
{
    debugger: D,
    sqvm: Box<SquirrelVM>,
    _this: PhantomData<&'a T>
}

impl<'a, D, T> Drop for SqCrab<'a, D, T>
where D: ScriptDebugger,
      &'a mut T: CanSquirrel<Into = UserPointer<&'a mut T>>
{
    fn drop(&mut self) {
    }
}

impl<'a, T> SqCrab<'a, CrabDebugger, T>
where &'a mut T: CanSquirrel<Into = UserPointer<&'a mut T>>
{
    pub fn new() -> SqCrabBuilder<'a, T> {
        SqCrabBuilder::<T>::new(DebuggerFlags::default())
    }
}

impl<'a, D, T> SqCrab<'a, D, T>
where D: ScriptDebugger,
      &'a mut T: CanSquirrel<Into = UserPointer<&'a mut T>>
{
    pub fn from_parts(sqvm: Box<SquirrelVM>, debugger: D) -> Self {
        Self { debugger, sqvm, _this: PhantomData::<&'a T> }
    }
    pub fn debugger(&self) -> &D {
        &self.debugger
    }
    pub fn debugger_mut(&mut self) -> &mut D {
        &mut self.debugger
    }
    pub fn register<F>(&mut self) -> Result<(), SquirrelError>
    where F: DomainRegistrar {
        F::add_functions(self)
    }

    /*
    pub fn using_this<F>(&mut self, p: &'a mut T, cb: F) -> Result<(), Box<dyn Error>>
    where F: Fn(&mut Self) -> Result<(), Box<dyn Error>> {
        unsafe { self.sqvm.set_this(p) };
        cb(self)?;
        unsafe { self.sqvm.clear_this() };
        Ok(())
    }
    */
    /*
    pub fn get_this(&self) -> Result<&mut T, SquirrelError> {
        // unsafe { &mut *(squirrel::squirrel_sys::bindings::root::sq_getforeignptr(self.raw()) as *mut T) }
        // self.sqvm.get_this()
        Err(SquirrelError::ForeignPointerNotSet)
    }
    */
    pub fn using_this<F>(&mut self, p: *mut T, cb: F) -> Result<(), Box<dyn Error>>
    where F: Fn(&mut Self) -> Result<(), Box<dyn Error>> {
        unsafe { self.sqvm.set_this(std::mem::transmute::<_, &'a mut T>(p)) };
        cb(self)?;
        unsafe { self.sqvm.clear_this() };
        Ok(())
    }
}

impl<'a, D, T> Deref for SqCrab<'a, D, T>
where D: ScriptDebugger,
      &'a mut T: CanSquirrel<Into = UserPointer<&'a mut T>>
{
    type Target = SquirrelVM;

    fn deref(&self) -> &Self::Target {
        self.sqvm.as_ref()
    }
}

impl<'a, D, T> DerefMut for SqCrab<'a, D, T>
where D: ScriptDebugger,
      &'a mut T: CanSquirrel<Into = UserPointer<&'a mut T>>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.sqvm.as_mut()
    }
}