use std::error::Error;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::Path;
use squirrel_sys::bindings::root::*;
use crate::err::SquirrelError;
use crate::obj_type::SquirrelObject;
use crate::type_cnv::CanSquirrel;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct ThreadSafeSquirrelVMPointer(pub(crate) HSQUIRRELVM);
impl Deref for ThreadSafeSquirrelVMPointer {
    type Target = HSQUIRRELVM;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Hash for ThreadSafeSquirrelVMPointer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0 as usize).hash(state)
    }
}
unsafe impl Send for ThreadSafeSquirrelVMPointer {}
unsafe impl Sync for ThreadSafeSquirrelVMPointer {}

#[derive(Debug)]
pub struct SquirrelVMBuilder {
    stack_size: usize,
    print_cb: Option<fn(&str)>,
    error_cb: Option<fn(&str)>,
    enable_debug_info: bool,
    notify_all_exceptions: bool,
    compile_error_cb: Option<crate::print_cb::CompilerErrorCallback>,
    debug_hook_cb: Option<crate::print_cb::DebugHookCallback>,
}

impl Default for SquirrelVMBuilder {
    fn default() -> Self {
        Self {
            // vm options
            stack_size: 0x400,
            print_cb: None,
            error_cb: None,
            // compiler options
            enable_debug_info: false,
            notify_all_exceptions: false,
            compile_error_cb: None,
            debug_hook_cb: None
        }
    }
}

impl SquirrelVMBuilder {
    pub fn set_stack_size(mut self, v: usize) -> Self {
        self.stack_size = v;
        self
    }
    pub fn set_print_fn(mut self, cb: fn(&str)) -> Self {
        self.print_cb = Some(cb);
        self
    }
    pub fn set_error_fn(mut self, cb: fn(&str)) -> Self {
        self.error_cb = Some(cb);
        self
    }
    pub fn set_enable_debug_info(mut self, v: bool) -> Self {
        self.enable_debug_info = v;
        self
    }
    pub fn set_notify_all_exceptions(mut self, v: bool) -> Self {
        self.notify_all_exceptions = v;
        self
    }
    pub fn set_compile_error_cb(mut self, cb: crate::print_cb::CompilerErrorCallback) -> Self {
        self.compile_error_cb = Some(cb);
        self
    }
    pub fn set_debug_hook_cb(mut self, cb: crate::print_cb::DebugHookCallback) -> Self {
        self.debug_hook_cb = Some(cb);
        self
    }

    pub fn build(self) -> SquirrelVM {
        let handle = unsafe { sq_open((self.stack_size as i64).into()) };
        if let Some(cb) = self.print_cb {
            crate::print_cb::register_print_format_callback(handle, cb);
        }
        if let Some(cb) = self.error_cb {
            crate::print_cb::register_error_format_callback(handle, cb);
        }
        if let Some(cb) = self.compile_error_cb {
            crate::print_cb::register_compiler_error_callback(handle, cb);
        }
        if let Some(cb) = self.debug_hook_cb {
            crate::print_cb::register_debug_hook_callback(handle, cb);
        }
        unsafe {
            sq_setprintfunc(
                handle, Some(crate::print_cb::sq_print_callback_cpp),
                Some(crate::print_cb::sq_error_callback_cpp)
            );
            sq_setcompilererrorhandler(handle, Some(crate::print_cb::sq_compile_error_callback));
            sq_setnativedebughook(handle, Some(crate::print_cb::sq_debug_hook_callback));
            sq_enabledebuginfo(handle, self.enable_debug_info.into_squirrel());
            sq_notifyallexceptions(handle, self.notify_all_exceptions.into_squirrel());
        }
        SquirrelVM { handle }
    }
}

#[macro_export]
macro_rules! sqvm_call_push_param {
    ($vm:ident) => {};
    ($vm:ident $val:expr, $ty:ty) => {
        $vm.push::<$ty>(&$val);
    };
    ($vm:ident $hval:expr, $hty:ty, $($val:expr, $ty:ty),*) => {
        $crate::sqvm_call_push_param!($vm $hval, $hty);
        $crate::sqvm_call_push_param!($vm $($val, $ty),*);
    };
}

#[macro_export]
macro_rules! sqvm_call_replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

#[macro_export]
macro_rules! sqvm_call_count_args {
    ($($t:tt)*) => { <[()]>::len(&[$($crate::sqvm_call_replace_expr!($t ())),*])};
}

#[macro_export]
macro_rules! sqvm_call {

    ($vm:ident $name:ident($($val:expr, $ty:ty),* $(,)?) -> $ret:ty) => {
        {
            let n = stringify!($name);
            let handle: ::squirrel_sys::bindings::root::HSQUIRRELVM = unsafe { $vm.raw() };
            unsafe {
               ::squirrel_sys::bindings::root::sq_pushroottable(handle);
               ::squirrel_sys::bindings::root::sq_pushstring(handle, n.as_ptr() as _, n.len() as _);
               ::squirrel_sys::bindings::root::sq_get(handle, -2); // get function from root table
               ::squirrel_sys::bindings::root::sq_push(handle, -2); // root table
               let args = (1 + $crate::sqvm_call_count_args!($($ty)*)) as i64;
               $crate::sqvm_call_push_param!($vm $($val, $ty),*);
               let res = ::squirrel_sys::bindings::root::sq_call(handle, args, true.into(), true.into());
               let val: $ret = $vm.get::<$ret>(1)?;
               ::squirrel_sys::bindings::root::sq_pop(handle, 3);
               Ok::<$ret, $crate::err::SquirrelError>(val)
            }
        }
    };
}

// const SQ_VMSTATE_IDLE         : i64 = 0;
// const SQ_VMSTATE_RUNNING      : i64 = 1;
const SQ_VMSTATE_SUSPENDED    : i64 = 2;

#[derive(Debug)]
pub struct SquirrelVM {
    pub(crate) handle: HSQUIRRELVM,
}

impl SquirrelVM {
    pub fn new() -> SquirrelVMBuilder {
        SquirrelVMBuilder::default()
    }

    pub fn version() -> (u32, u32) {
        let raw = unsafe { sq_getversion() };
        ((raw / 100) as u32, (raw % 100) as u32)
    }

    pub unsafe fn raw(&self) -> HSQUIRRELVM {
        self.handle
    }

    // push
    pub fn push<T>(&mut self, value: &T) where T: CanSquirrel {
        T::push(value, self);
    }

    // pop
    pub fn pop_top(&mut self) {
        unsafe { sq_poptop(self.handle) }
    }

    pub fn pop(&mut self, n: usize) {
        unsafe { sq_pop(self.handle, n as _) }
    }

    // get
    pub fn get<T>(&self, index: usize) -> Result<T, SquirrelError> where T: CanSquirrel {
        T::get(self, index).map(|v| T::from_squirrel(v))
    }

    // get_type

    // call
    pub fn call<T: CanSquirrel>(&self, name: &str, params: usize) -> Result<T, SquirrelError> {
        unsafe {
            sq_pushroottable(self.handle);
            sq_pushstring(self.handle, name.as_ptr() as _, name.len() as _);
            sq_get(self.handle, -2); // get function from root table
            sq_push(self.handle, -2); // root table
            let res = sq_call(self.handle, 1 + params as i64, true.into(), true.into());
            let val  = self.get::<T>(1)?;
            sq_pop(self.handle, 2 + (params as i64));
            Ok(val)
        }

    }

    pub fn get_stack_len(&mut self) -> usize {
        -unsafe { sq_gettop(self.handle) } as _
    }

    fn try_compile(&self, bytes: &str, path: &str) -> Result<(), SquirrelError> {
        let res = unsafe { sq_compilebuffer(self.handle, bytes.as_ptr() as _, bytes.len() as i64, path.as_ptr() as _, true.into()) };
        match res {
            0 => Ok(()),
            _ => Err(SquirrelError::CouldNotCompileBuffer)
        }
    }

    fn source_name(&self) -> String {
        format!("SQVM @ 0x{:x}", self.handle as usize)
    }

    fn import_text_inner(&mut self, buf: &str, src: &str) -> Result<(), SquirrelError> {
        self.try_compile(buf, src)?;
        unsafe {
            // call main to import functions
            sq_pushroottable(self.handle);
            sq_call(self.handle, 1, false.into(), true.into());
            sq_poptop(self.handle);
        }
        Ok(())
    }

    /// Compiles a squirrel program (.nut) from the given file path.
    pub fn import_text_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let path_str = path.as_ref().to_str().unwrap();
        let bytes = std::fs::read_to_string(path.as_ref())?;
        self.import_text_inner(&bytes, path_str)?;
        Ok(())
    }

    /// Compiles a squirrel program (.nut) from the string.
    pub fn import_text_from_str(&mut self, buf: &str) -> Result<(), SquirrelError> {
        self.import_text_inner(buf, &self.source_name())
    }

    // file reading
    pub fn import_binary_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn import_binary_from_slice(&mut self, buf: &[u8]) -> Result<(), SquirrelError> {
        Ok(())
    }

    // suspend/wakeup
    /*
    pub fn suspend(&mut self) -> bool {
        let state = unsafe { sq_getvmstate(self.0) } as i64;
        if state != SQ_VMSTATE_SUSPENDED {
            (unsafe { sq_suspendvm(self.0) } as i64) != 0
        } else {
            false
        }
    }
    */
}

impl Drop for SquirrelVM {
    fn drop(&mut self) {
        if crate::print_cb::get_print_format_callback(self.handle).is_some() {
            crate::print_cb::remove_print_format_callback(self.handle);
        }
        if crate::print_cb::get_error_format_callback(self.handle).is_some() {
            crate::print_cb::remove_error_format_callback(self.handle);
        }
        if crate::print_cb::get_compiler_error_callback(self.handle).is_some() {
            crate::print_cb::remove_compiler_error_callback(self.handle);
        }
        if crate::print_cb::get_debug_hook_callback(self.handle).is_some() {
            crate::print_cb::remove_debug_hook_callback(self.handle);
        }
        unsafe { sq_close(self.handle) };
    }
}