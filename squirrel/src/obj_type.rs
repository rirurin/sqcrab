use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
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

impl SquirrelObject for String {
    fn push(&self, vm: &mut SquirrelVM) {
        unsafe { sq_pushstring(vm.handle, self.as_ptr() as _, self.len() as i64) };
    }

    fn get(vm: &SquirrelVM, index: usize) -> Result<Self, SquirrelError> {
        let mut out_str = std::ptr::null();
        let res = unsafe { sq_getstring(vm.handle, -(index as i64), &mut out_str) };
        if res != 0 { return Err(SquirrelError::GetWrongObjectType) }
        let out_str = unsafe { std::ffi::CStr::from_ptr(out_str).to_str()
            .map_err(|e| SquirrelError::Utf8Error(e))? };
        Ok(out_str.to_string())
    }
}

pub struct UserPointer<T>(NonNull<T>);

impl<T> UserPointer<T> {
    pub fn new(v: &T) -> Self {
        Self(unsafe { NonNull::new_unchecked(&raw const *v as *mut T) })
    }
}

impl<T> Deref for UserPointer<T> {
    type Target = NonNull<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for UserPointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<UserPointer<T>> for SQUserPointer {
    fn from(value: UserPointer<T>) -> Self {
        value.0.as_ptr() as _
    }
}

impl<'a, T> From<&'a UserPointer<T>> for SQUserPointer {
    fn from(value: &'a UserPointer<T>) -> Self {
        value.0.as_ptr() as _
    }
}
impl<T> From<SQUserPointer> for UserPointer<T> {
    fn from(value: SQUserPointer) -> Self {
        UserPointer(unsafe { NonNull::new_unchecked(value as _) })
    }
}

impl<T> SquirrelObject for UserPointer<T> {
    fn push(&self, vm: &mut SquirrelVM) {
        unsafe { sq_pushuserpointer(vm.handle, self.into()) };
    }

    fn get(vm: &SquirrelVM, index: usize) -> Result<Self, SquirrelError> {
        let mut ptr: SQUserPointer = std::ptr::null_mut();
        let res = unsafe { sq_getuserpointer(vm.handle, -(index as i64), &mut ptr) };
        if res != 0 { return Err(SquirrelError::GetWrongObjectType) }
        Ok(ptr.into())
    }
}