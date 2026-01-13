use std::marker::PhantomData;
use std::mem::MaybeUninit;
use squirrel_sys::bindings::root::*;
use crate::err::SquirrelError;
use crate::obj_type::{SquirrelObject, UserPointer};
use crate::vm::SquirrelVM;

pub trait SquirrelTypeId where Self: Sized {
    fn type_id() -> u32;
}

pub struct UserObject<T> {
    handle: Box<HSQOBJECT>,
    // for drop
    _owner: HSQUIRRELVM,
    _type: PhantomData<T>
}

impl<T> UserObject<T> where T: SquirrelTypeId {
    fn push(&self, vm: &mut SquirrelVM) {
        todo!()
    }

    fn get(vm: &SquirrelVM, index: usize) -> Result<Self, SquirrelError> {
        let mut out_obj = MaybeUninit::uninit();
        let res = unsafe { sq_getstackobj(vm.handle, -(index as i64), out_obj.as_mut_ptr()) };
        if res != 0 { return Err(SquirrelError::GetWrongObjectType) }
        let mut handle = Box::new(unsafe { out_obj.assume_init() });
        if handle._type != T::type_id() as _ { return Err(SquirrelError::ObjectTypeDoesNotMatch) }
        unsafe { sq_addref(vm.handle, handle.as_mut()) };
        Ok(Self { handle, _owner: vm.handle, _type: PhantomData::<T> })
    }
}

impl<T> Drop for UserObject<T> {
    fn drop(&mut self) {
        unsafe { sq_release(self._owner, self.handle.as_mut()) };
    }
}

pub struct Table;

impl SquirrelTypeId for Table {
    fn type_id() -> u32 {
        tagSQObjectType_OT_TABLE as _
    }
}

pub struct Array;

impl SquirrelTypeId for Array {
    fn type_id() -> u32 {
        tagSQObjectType_OT_ARRAY as _
    }
}

/*
impl<'a> From<&'a UserObject<SQBool>> for SQBool {
    fn from(value: &'a UserObject<SQBool>) -> Self {
        unsafe { sq_objtobool(value.handle.as_ref()) }
    }
}

impl<'a> From<&'a UserObject<SQFloat>> for SQFloat {
    fn from(value: &'a UserObject<SQFloat>) -> Self {
        unsafe { sq_objtofloat(value.handle.as_ref()) }
    }
}

impl<'a> From<&'a UserObject<SQInteger>> for SQInteger {
    fn from(value: &'a UserObject<SQInteger>) -> Self {
        unsafe { sq_objtointeger(value.handle.as_ref()) }
    }
}

impl<'a> From<&'a UserObject<String>> for String {
    fn from(value: &'a UserObject<String>) -> Self {
        let value = unsafe { sq_objtostring(value.handle.as_ref()) };
        unsafe { std::ffi::CStr::from_ptr(value).to_str().unwrap().to_owned() }
    }
}

impl<'a, T> From<&'a UserObject<SQUserPointer>> for UserPointer<T> {
    fn from(value: &'a UserObject<SQUserPointer>) -> Self {
        UserPointer::<T>::new(unsafe { &*(sq_objtouserpointer(value.handle.as_ref()) as *const T) })
    }
}
*/

impl<'a> TryFrom<&'a UserObject<SQBool>> for SQBool {
    type Error = SquirrelError;

    fn try_from(value: &'a UserObject<SQBool>) -> Result<Self, Self::Error> {
        Ok(unsafe { sq_objtobool(value.handle.as_ref()) })
    }
}

impl<'a> TryFrom<&'a UserObject<SQFloat>> for SQFloat {
    type Error = SquirrelError;

    fn try_from(value: &'a UserObject<SQFloat>) -> Result<Self, Self::Error> {
        Ok(unsafe { sq_objtofloat(value.handle.as_ref()) })
    }
}

impl<'a> TryFrom<&'a UserObject<SQInteger>> for SQInteger {
    type Error = SquirrelError;

    fn try_from(value: &'a UserObject<SQInteger>) -> Result<Self, Self::Error> {
        Ok(unsafe { sq_objtointeger(value.handle.as_ref()) })
    }
}

impl<'a> TryFrom<&'a UserObject<String>> for String {
    type Error = SquirrelError;

    fn try_from(value: &'a UserObject<String>) -> Result<Self, Self::Error> {
        let value = unsafe { sq_objtostring(value.handle.as_ref()) };
        Ok(unsafe { std::ffi::CStr::from_ptr(value).to_str()
            .map_err(|e| SquirrelError::Utf8Error(e))?.to_owned() })
    }
}

impl<'a, T> TryFrom<&'a UserObject<SQUserPointer>> for UserPointer<T> {
    type Error = SquirrelError;

    fn try_from(value: &'a UserObject<SQUserPointer>) -> Result<Self, Self::Error> {
        Ok(UserPointer::<T>::new(unsafe { &*(sq_objtouserpointer(value.handle.as_ref()) as *const T) }))
    }
}