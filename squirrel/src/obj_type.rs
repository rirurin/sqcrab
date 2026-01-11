use squirrel_sys::bindings::root::*;
use crate::err::SquirrelError;
use crate::vm::SquirrelVM;

pub trait SquirrelObject where Self: Sized {
    fn push(&self, vm: &mut SquirrelVM);
    fn get(vm: &SquirrelVM, index: usize) -> Result<Self, SquirrelError>;
}

impl SquirrelObject for SQInteger {
    fn push(&self, vm: &mut SquirrelVM) {
        unsafe { sq_pushinteger(vm.handle, *self) };
    }
    fn get(vm: &SquirrelVM, index: usize) -> Result<Self, SquirrelError> {
        let mut value: Self = 0;
        let res = unsafe { sq_getinteger(vm.handle, -(index as i64), &mut value) };
        if res != 0 { return Err(SquirrelError::GetWrongObjectType) }
        Ok(value)
    }
}

impl SquirrelObject for SQFloat {
    fn push(&self, vm: &mut SquirrelVM) {
        unsafe { sq_pushfloat(vm.handle, *self) };
    }
    fn get(vm: &SquirrelVM, index: usize) -> Result<Self, SquirrelError> {
        let mut value: Self = 0.;
        let res = unsafe { sq_getfloat(vm.handle, -(index as i64), &mut value) };
        if res != 0 { return Err(SquirrelError::GetWrongObjectType) }
        Ok(value)
    }
}

impl SquirrelObject for SQBool {
    fn push(&self, vm: &mut SquirrelVM) {
        unsafe { sq_pushbool(vm.handle, *self) };
    }
    fn get(vm: &SquirrelVM, index: usize) -> Result<Self, SquirrelError> {
        let mut value: Self = false.into();
        let res = unsafe { sq_getbool(vm.handle, -(index as i64), &mut value) };
        if res != 0 { return Err(SquirrelError::GetWrongObjectType) }
        Ok(value)
    }
}

impl SquirrelObject for () {
    fn push(&self, vm: &mut SquirrelVM) {
        unsafe { sq_pushnull(vm.handle) };
    }
    fn get(_: &SquirrelVM, _: usize) -> Result<Self, SquirrelError> {
        Ok(())
    }
}