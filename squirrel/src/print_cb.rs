use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use squirrel_sys::bindings::root::*;
use crate::vm::ThreadSafeSquirrelVMPointer;
// print/error

type CallbackTypeBase<T> = Option<HashMap<ThreadSafeSquirrelVMPointer, T>>;

static PRINT_FORMAT_CALLBACKS: Mutex<CallbackTypeBase<fn(&str)>> = Mutex::new(None);
static ERROR_FORMAT_CALLBACKS: Mutex<CallbackTypeBase<fn(&str)>> = Mutex::new(None);

fn get_print_format_callbacks<T>(map: &'static Mutex<CallbackTypeBase<T>>)
                              -> MutexGuard<'static, CallbackTypeBase<T>> {
    let mut callbacks = map.lock().unwrap();
    if callbacks.is_none() {
        *callbacks = Some(HashMap::new());
    }
    callbacks
}

pub(crate) fn register_print_format_callback(vm: HSQUIRRELVM, cb: fn(&str)) {
    get_print_format_callbacks(&PRINT_FORMAT_CALLBACKS).as_mut().unwrap()
        .insert(ThreadSafeSquirrelVMPointer(vm), cb);
}

pub(crate) fn remove_print_format_callback(vm: HSQUIRRELVM) {
    get_print_format_callbacks(&PRINT_FORMAT_CALLBACKS).as_mut().unwrap()
        .remove(&ThreadSafeSquirrelVMPointer(vm));
}

pub(crate) fn get_print_format_callback(vm: HSQUIRRELVM) -> Option<fn(&str)> {
    get_print_format_callbacks(&PRINT_FORMAT_CALLBACKS).as_ref().unwrap()
        .get(&ThreadSafeSquirrelVMPointer(vm)).map(|v| *v)
}

pub(crate) fn register_error_format_callback(vm: HSQUIRRELVM, cb: fn(&str)) {
    get_print_format_callbacks(&ERROR_FORMAT_CALLBACKS).as_mut().unwrap()
        .insert(ThreadSafeSquirrelVMPointer(vm), cb);
}

pub(crate) fn remove_error_format_callback(vm: HSQUIRRELVM) {
    get_print_format_callbacks(&ERROR_FORMAT_CALLBACKS).as_mut().unwrap()
        .remove(&ThreadSafeSquirrelVMPointer(vm));
}

pub(crate) fn get_error_format_callback(vm: HSQUIRRELVM) -> Option<fn(&str)> {
    get_print_format_callbacks(&ERROR_FORMAT_CALLBACKS).as_ref().unwrap()
        .get(&ThreadSafeSquirrelVMPointer(vm)).map(|v| *v)
}

#[link(name = "squirrel_print_format")]
unsafe extern "C" {
    pub(crate) fn sq_print_callback_cpp(vm: HSQUIRRELVM, fmt: *const SQChar, ...);
    pub(crate) fn sq_error_callback_cpp(vm: HSQUIRRELVM, fmt: *const SQChar, ...);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sq_print_callback_rust(vm: HSQUIRRELVM, str: *const SQChar) {
    let str = unsafe { std::ffi::CStr::from_ptr(str).to_str().unwrap() };
    if let Some(cb) = get_print_format_callback(vm) {
        cb(str);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sq_error_callback_rust(vm: HSQUIRRELVM, str: *const SQChar) {
    let str = unsafe { std::ffi::CStr::from_ptr(str).to_str().unwrap() };
    if let Some(cb) = get_error_format_callback(vm) {
        cb(str);
    }
}

// typedef void (*SQCOMPILERERROR)(HSQUIRRELVM,const SQChar * /*desc*/,const SQChar * /*source*/,SQInteger /*line*/,SQInteger /*column*/);
pub(crate) type CompilerErrorCallback = fn(&str, &str, i64, i64);

static COMPILER_ERROR_CALLBACKS: Mutex<CallbackTypeBase<CompilerErrorCallback>> = Mutex::new(None);

pub(crate) fn register_compiler_error_callback(vm: HSQUIRRELVM, cb: CompilerErrorCallback) {
    get_print_format_callbacks(&COMPILER_ERROR_CALLBACKS).as_mut().unwrap()
        .insert(ThreadSafeSquirrelVMPointer(vm), cb);
}

pub(crate) fn remove_compiler_error_callback(vm: HSQUIRRELVM) {
    get_print_format_callbacks(&COMPILER_ERROR_CALLBACKS).as_mut().unwrap()
        .remove(&ThreadSafeSquirrelVMPointer(vm));
}

pub(crate) fn get_compiler_error_callback(vm: HSQUIRRELVM) -> Option<CompilerErrorCallback> {
    get_print_format_callbacks(&COMPILER_ERROR_CALLBACKS).as_ref().unwrap()
        .get(&ThreadSafeSquirrelVMPointer(vm)).map(|v| *v)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn sq_compile_error_callback(vm: HSQUIRRELVM, desc: *const SQChar, source: *const SQChar, line: SQInteger, column: SQInteger) {
    let desc = unsafe { std::ffi::CStr::from_ptr(desc).to_str().unwrap() };
    let source = unsafe { std::ffi::CStr::from_ptr(source).to_str().unwrap() };
    if let Some(cb) = get_compiler_error_callback(vm) {
        cb(desc, source, line, column);
    }
}

// debug hook

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum DebugHookType {
    CallFunc,
    ExecLine,
    RetFunc,
    Unknown
}

impl From<i64> for DebugHookType {
    fn from(value: i64) -> Self {
        match value {
            0x63 => Self::CallFunc,
            0x6c => Self::ExecLine,
            0x72 => Self::RetFunc,
            _ => Self::Unknown
        }
    }
}

// typedef void (*SQDEBUGHOOK)(HSQUIRRELVM /*v*/, SQInteger /*type*/, const SQChar * /*sourcename*/, SQInteger /*line*/, const SQChar * /*funcname*/);
pub(crate) type DebugHookCallback = fn(DebugHookType, &str, i64, &str);

static DEBUG_HOOK_CALLBACKS: Mutex<CallbackTypeBase<DebugHookCallback>> = Mutex::new(None);

pub(crate) fn register_debug_hook_callback(vm: HSQUIRRELVM, cb: DebugHookCallback) {
    get_print_format_callbacks(&DEBUG_HOOK_CALLBACKS).as_mut().unwrap()
        .insert(ThreadSafeSquirrelVMPointer(vm), cb);
}

pub(crate) fn remove_debug_hook_callback(vm: HSQUIRRELVM) {
    get_print_format_callbacks(&DEBUG_HOOK_CALLBACKS).as_mut().unwrap()
        .remove(&ThreadSafeSquirrelVMPointer(vm));
}

pub(crate) fn get_debug_hook_callback(vm: HSQUIRRELVM) -> Option<DebugHookCallback> {
    get_print_format_callbacks(&DEBUG_HOOK_CALLBACKS).as_ref().unwrap()
        .get(&ThreadSafeSquirrelVMPointer(vm)).map(|v| *v)
}

#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn sq_debug_hook_callback(vm: HSQUIRRELVM, _type: SQInteger, sourcename: *const SQChar, line: SQInteger, funcname: *const SQChar) {
    let sourcename = unsafe { std::ffi::CStr::from_ptr(sourcename).to_str().unwrap() };
    let funcname = unsafe { std::ffi::CStr::from_ptr(funcname).to_str().unwrap() };
    if let Some(cb) = get_debug_hook_callback(vm) {
        cb(_type.into(), sourcename, line, funcname);
    }
}