use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::io::{Cursor, Read};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::ptr::NonNull;
use std::sync::{Mutex, MutexGuard};
use squirrel_sys::bindings::root::*;
use crate::err::SquirrelError;
use crate::obj_type::SquirrelObject;
use crate::type_cnv::CanSquirrel;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ThreadSafeSquirrelVMPointer(pub(crate) HSQUIRRELVM);
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

// Returns the number of parameters that were pushed (sq_pushx)
type SquirrelFunction = fn(&mut SquirrelVM) -> SQInteger;

pub struct SQVMNativeFunctionLink {
    pub(crate) sqvm: NonNull<SquirrelVM>,
    pub(crate) functions: HashMap<String, SquirrelFunction>
}

impl SQVMNativeFunctionLink {
    pub(crate) fn new(handle: &mut SquirrelVM) -> Self {
        let handle = unsafe { NonNull::new_unchecked(&raw mut *handle) };
        Self { sqvm: handle, functions: HashMap::new() }
    }
}

impl Deref for SQVMNativeFunctionLink {
    type Target = SquirrelVM;
    fn deref(&self) -> &Self::Target {
        unsafe { self.sqvm.as_ref() }
    }
}

impl DerefMut for SQVMNativeFunctionLink {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.sqvm.as_mut() }
    }
}

impl Hash for SQVMNativeFunctionLink {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.sqvm.as_ptr() as usize).hash(state)
    }
}
unsafe impl Send for SQVMNativeFunctionLink {}
unsafe impl Sync for SQVMNativeFunctionLink {}

// Used to obtain a reference to SquirrelVM from the HSQUIRRELVM ptr in SQFUNCTION

type SquirrelHandleMap = HashMap<ThreadSafeSquirrelVMPointer, SQVMNativeFunctionLink>;

static SQUIRREL_HANDLE_INSTANCES: Mutex<Option<SquirrelHandleMap>> = Mutex::new(None);

pub(crate) fn get_squirrel_handle_instances() -> MutexGuard<'static, Option<SquirrelHandleMap>> {
    let mut instances = SQUIRREL_HANDLE_INSTANCES.lock().unwrap();
    if instances.is_none() {
        *instances = Some(HashMap::new());
    }
    instances
}

pub(crate) unsafe fn add_squirrel_handle(vm: &mut SquirrelVM) {
    get_squirrel_handle_instances().as_mut().unwrap().insert(
        ThreadSafeSquirrelVMPointer(vm.handle),
        SQVMNativeFunctionLink::new(vm)
    );
}

pub(crate) unsafe fn check_squirrel_handle(vm: &SquirrelVM) -> bool {
    get_squirrel_handle_instances().as_ref().unwrap()
        .contains_key(&ThreadSafeSquirrelVMPointer(vm.handle))
}

pub(crate) unsafe fn add_squirrel_function_link(vm: &mut SquirrelVM, name: &str, func: SquirrelFunction) {
    let mut instances = get_squirrel_handle_instances();
    if let Some(link) = instances.as_mut().unwrap()
        .get_mut(&ThreadSafeSquirrelVMPointer(vm.handle)) {
        link.functions.insert(name.to_string(), func);
    }
}

pub(crate) unsafe fn remove_squirrel_handle(handle: HSQUIRRELVM) {
    get_squirrel_handle_instances().as_mut().unwrap().remove(&ThreadSafeSquirrelVMPointer(handle));
}

unsafe extern "C" fn sq_function_base(handle: HSQUIRRELVM) -> SQInteger {
    // this requires that functions are registered with a name using sq_setnativeclosurename
    let mut info: MaybeUninit<SQStackInfos> = MaybeUninit::uninit();
    unsafe { sq_stackinfos(handle, 0, info.as_mut_ptr()) };
    let info = unsafe { info.assume_init() };
    if info.funcname == std::ptr::null() { return 0; }
    let func_name = unsafe { std::ffi::CStr::from_ptr(info.funcname).to_str().unwrap() };
    let mut instances = get_squirrel_handle_instances();
    let handle_info = instances.as_mut()
        .unwrap().get(&ThreadSafeSquirrelVMPointer(handle));
    if handle_info.is_none() { return 0; }
    let handle_info = handle_info.unwrap();
    let sqvm = unsafe { &mut *handle_info.sqvm.as_ptr() };
    let rust_func = handle_info.functions.get(func_name);
    if rust_func.is_none() { return 0; }
    let rust_func = *rust_func.unwrap();
    // drop SQUIRREL_HANDLE_INSTANCES so other threads can run Squirrel scripts at the same time
    drop(instances);
    rust_func(sqvm)
}

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
macro_rules! sqvm_call_replace_type {
    ($_t:ty) => (());
}

#[macro_export]
macro_rules! sqvm_call_count_type_args {
    ($($t:ty),*) => { <[()]>::len(&[$($crate::sqvm_call_replace_type!($t)),*])};
}

#[macro_export]
macro_rules! squirrel {
    ($vm:ident $name:ident($($val:expr, $ty:ty),* $(,)?)) => {
        $crate::squirrel!($vm $name($($val, $ty),*) -> ())
    };
    ($vm:ident $name:ident($($val:expr, $ty:ty),* $(,)?) -> $ret:ty) => {
        {
            let n = stringify!($name);
            let handle: ::squirrel_sys::bindings::root::HSQUIRRELVM = unsafe { $vm.raw() };
            unsafe {
               ::squirrel_sys::bindings::root::sq_pushroottable(handle);
               ::squirrel_sys::bindings::root::sq_pushstring(handle, n.as_ptr() as _, n.len() as _);
               ::squirrel_sys::bindings::root::sq_get(handle, -2); // get function from root table
               ::squirrel_sys::bindings::root::sq_push(handle, -2); // root table
               let args = (1 + $crate::sqvm_call_count_type_args!($($ty),*)) as i64;
               $crate::sqvm_call_push_param!($vm $($val, $ty),*);
               let res = ::squirrel_sys::bindings::root::sq_call(handle, args, true.into(), true.into());
               match res {
                   0 => {
                       let val: $ret = $vm.get::<$ret>(1)?;
                       ::squirrel_sys::bindings::root::sq_pop(handle, 3);
                       Ok::<$ret, $crate::err::SquirrelError>(val)
                   },
                   _ => Err($crate::err::SquirrelError::ErrorWhileCalling)
               }
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
    /*
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
    */

    pub fn get_stack_len(&mut self) -> usize {
        -unsafe { sq_gettop(self.handle) } as _
    }

    fn try_compile(&self, bytes: &str, path: &str) -> Result<(), SquirrelError> {
        match unsafe { sq_compilebuffer(self.handle, bytes.as_ptr() as _, bytes.len() as i64, path.as_ptr() as _, true.into()) } {
            0 => Ok(()),
            _ => Err(SquirrelError::CouldNotCompileSource)
        }
    }

    unsafe extern "C" fn read_stream(up: SQUserPointer, out: SQUserPointer, mut size: SQInteger) -> SQInteger {
        let pcursor = unsafe { &mut *(up as *mut Cursor<&[u8]>) };
        let plen = pcursor.get_ref().len();
        if pcursor.position() + (size as u64) >= plen as u64 {
            size = plen as i64 - pcursor.position() as i64;
        }
        if plen > 0 {
            let buf = unsafe { std::slice::from_raw_parts_mut(out as *mut u8, size as _) };
            pcursor.read_exact(buf).unwrap();
            size
        } else {
            -1
        }
    }

    fn try_read(&self, buf: &[u8], path: &str) -> Result<(), SquirrelError> {
        let mut cursor = Cursor::new(buf);
        match unsafe { sq_readclosure(self.handle, Some(Self::read_stream), &raw mut cursor as _)} {
            0 => Ok(()),
            _ => Err(SquirrelError::CouldNotReadBytecode)
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
        let buf = std::fs::read_to_string(path.as_ref())?;
        self.import_text_inner(&buf, path_str)?;
        Ok(())
    }

    /// Compiles a squirrel program (.nut) from the string.
    pub fn import_text_from_str(&mut self, buf: &str) -> Result<(), SquirrelError> {
        self.import_text_inner(buf, &self.source_name())
    }

    fn import_binary_inner(&mut self, buf: &[u8], src: &str) -> Result<(), SquirrelError> {
        self.try_read(buf, src)?;
        unsafe {
            // call main to import functions
            sq_pushroottable(self.handle);
            sq_call(self.handle, 1, false.into(), true.into());
            sq_poptop(self.handle);
        }
        Ok(())
    }

    // file reading
    pub fn import_binary_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let path_str = path.as_ref().to_str().unwrap();
        let buf = std::fs::read(path.as_ref())?;
        self.import_binary_inner(&buf, path_str)?;
        Ok(())
    }

    pub fn import_binary_from_slice(&mut self, buf: &[u8]) -> Result<(), SquirrelError> {
        self.import_binary_inner(&buf, &self.source_name())?;
        Ok(())
    }

    fn add_function_link(&mut self, name: &str, func: SquirrelFunction) {
        if unsafe { !check_squirrel_handle(self) } {
            unsafe { add_squirrel_handle(self) };
        }
        unsafe { add_squirrel_function_link(self, name, func) };
    }

    // add function
    pub fn add_function(&mut self, name: &str, func: SquirrelFunction) -> Result<(), SquirrelError> {
        unsafe {
            sq_pushroottable(self.handle);
            sq_pushstring(self.handle, name.as_ptr() as _, name.len() as _);
            sq_newclosure(self.handle, Some(sq_function_base), 0);
            let cname = name.as_bytes().last().map_or(
                name.to_owned(),
                |i| if *i != 0 { format!("{}\0", name) } else { name.to_owned() });
            let res = sq_setnativeclosurename(self.handle, -1, cname.as_ptr() as _);
            if res != 0 { return Err(SquirrelError::CouldNotSetNativeClosureName) }
            let res = sq_newslot(self.handle, -3, false.into());
            if res != 0 { return Err(SquirrelError::CouldNotAddFunction) }
            self.add_function_link(name, func);
            sq_poptop(self.handle);
            Ok(())
        }
    }

    pub unsafe fn add_function_raw(&mut self, name: &str, func: SQFUNCTION) -> Result<(), SquirrelError> {
        unsafe {
            sq_pushroottable(self.handle);
            sq_pushstring(self.handle, name.as_ptr() as _, name.len() as _);
            sq_newclosure(self.handle, func, 0);
            let res = sq_setnativeclosurename(self.handle, -1, name.as_ptr() as _);
            if res != 0 { return Err(SquirrelError::CouldNotSetNativeClosureName) }
            let res = sq_newslot(self.handle, -3, false.into());
            if res != 0 { return Err(SquirrelError::CouldNotAddFunction) }
            sq_poptop(self.handle);
            Ok(())
        }
    }

    // suspend/wakeup
    pub fn suspend(&mut self) -> Result<(), SquirrelError> {
        let state = unsafe { sq_getvmstate(self.handle) } as i64;
        let res = if state != SQ_VMSTATE_SUSPENDED {
            unsafe { sq_suspendvm(self.handle) }
        } else {
            -1
        };
        match res {
            0 => Ok(()),
            _ => Err(SquirrelError::CouldNotSuspendVM)
        }
    }

    pub fn wakeup(&mut self) -> Result<(), SquirrelError> {
        let state = unsafe { sq_getvmstate(self.handle) } as i64;
        let res = if state == SQ_VMSTATE_SUSPENDED {
            unsafe { sq_wakeupvm(self.handle, 0, 1, 1, 1)}
        } else {
            -1
        };
        match res {
            0 => Ok(()),
            _ => Err(SquirrelError::CouldNotWakeupVM)
        }
    }
}

impl Drop for SquirrelVM {
    fn drop(&mut self) {
        unsafe { remove_squirrel_handle(self.handle) };
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