use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use squirrel_sys::bindings::root::*;

#[derive(Debug)]
pub enum TestError {
    TestFileMissing
}

impl Error for TestError {}
impl Display for TestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

#[test]
fn check_version() -> Result<(), Box<dyn Error>> {
    assert_eq!(unsafe { sq_getversion() }, 320);
    Ok(())
}

#[test]
fn stack_insertion() -> Result<(), Box<dyn Error>> {
    let vm = unsafe { sq_open(0x400) };
    let sq_string = "crab";
    assert_eq!(unsafe { sq_gettop(vm) }, 0);
    unsafe {
        sq_pushinteger(vm, 64);
        sq_pushfloat(vm, 3.1);
        sq_pushbool(vm, true.into());
        sq_pushstring(vm, sq_string.as_ptr() as _, (sq_string.len() as i64).into());
    }
    assert_eq!(unsafe { sq_gettop(vm) }, 4);
    // get values back out
    unsafe {
        let mut out_str = std::ptr::null();
        sq_getstring(vm, -1, &mut out_str);
        assert_eq!(std::ffi::CStr::from_ptr(out_str).to_str()?, sq_string);
        sq_poptop(vm);
        let mut out_bool = false.into();
        sq_getbool(vm, -1, &mut out_bool);
        assert_eq!(out_bool, true.into());
        sq_poptop(vm);
        let mut float_out = 0.;
        sq_getfloat(vm, -1, &mut float_out);
        assert_eq!(float_out, 3.1);
        sq_poptop(vm);
        let mut int_out = 0;
        sq_getinteger(vm, -1, &mut int_out);
        assert_eq!(int_out, 64);
        sq_poptop(vm);
    }
    assert_eq!(unsafe { sq_gettop(vm) }, 0);
    unsafe { sq_close(vm); }
    Ok(())
}

unsafe extern "C" fn test_compiler_error(vm: HSQUIRRELVM, desc: *const SQChar, source: *const SQChar, line: SQInteger, column: SQInteger) {
    let desc = unsafe { std::ffi::CStr::from_ptr(desc).to_str().unwrap() };
    let source = unsafe { std::ffi::CStr::from_ptr(source).to_str().unwrap() };
    println!("Compile error: '{}' on '{}' @ {}:{}", desc, source, line, column);
}

fn test_compile(vm: HSQUIRRELVM, path: &str) -> Result<SQRESULT, Box<dyn Error>> {
    let test_print = std::env::current_dir()?.join(path);
    if !std::fs::exists(&test_print)? { return Err(Box::new(TestError::TestFileMissing)); }
    let sample_str = std::fs::read_to_string(test_print)?;
    Ok(unsafe { sq_compilebuffer(vm, sample_str.as_ptr() as _, sample_str.len() as i64, path.as_ptr() as _, true.into()) })
}

#[test]
fn compilation() -> Result<(), Box<dyn Error>> {
    let vm = unsafe { sq_open(0x400) };
    // path doesn't live long enough for it to be valid when the compiler error handler is called
    let good_path = "tests/data/sample_printf.nut";
    let bad_path = "tests/data/sample_printf_bad.nut";
    unsafe {
        sq_enabledebuginfo(vm, 1);
        // sq_setcompilererrorhandler(vm, Some(test_compiler_error));
        assert_eq!(test_compile(vm, good_path)?, 0);
        assert_eq!(sq_gettop(vm), 1);
        assert_eq!(test_compile(vm, bad_path)?, -1);
        assert_eq!(sq_gettop(vm), 1);
    }
    unsafe { sq_close(vm); }
    Ok(())
}

unsafe extern "C" fn test_debug_hook(vm: HSQUIRRELVM, event: SQInteger, source_name: *const SQChar, line: SQInteger, function: *const SQChar) {
    let source_name = unsafe { std::ffi::CStr::from_ptr(source_name).to_str().unwrap() };
    let function = unsafe { std::ffi::CStr::from_ptr(function).to_str().unwrap() };
    let fmt =  match event {
        0x63 => format!("debug: CALL '{}' @ {}:{}", function, source_name, line),
        0x6c => format!("debug: LINE '{}' @ {}:{}", function, source_name, line),
        0x72 => format!("debug: RET '{}' @ {}:{}", function, source_name, line),
        _ => format!("debug: UNK EVENT {}", event)
    };
    println!("{}", &fmt);
}

fn test_squirrel_function(vm: HSQUIRRELVM, name: &str, value: SQInteger) {
    unsafe {
        sq_pushroottable(vm);
        sq_pushstring(vm, name.as_ptr() as _, name.len() as _);
        sq_get(vm, -2); // get function from root table
        sq_pushroottable(vm);
        sq_call(vm, 1, true.into(), true.into());
        let mut out_int = 0;
        sq_getinteger(vm, -1, &mut out_int);
        assert_eq!(out_int, value);
        sq_pop(vm, 3);
    }
}

#[test]
fn call_squirrel_function() -> Result<(), Box<dyn Error>> {
    let vm = unsafe { sq_open(0x400) };
    let path = "tests/data/sample_functions.nut";
    let functions = ["test1", "test2", "test3"];
    let expected = [10, 20, 30];
    unsafe {
        sq_enabledebuginfo(vm, 1);
        // sq_setnativedebughook(vm, Some(test_debug_hook));
        test_compile(vm, path)?;
        sq_pushroottable(vm);
        sq_call(vm, 1, false.into(), true.into());
        sq_poptop(vm);
        for (func, value) in functions.iter().zip(expected.iter()) {
            test_squirrel_function(vm, *func, *value);
        }
    }
    unsafe { sq_close(vm); }
    Ok(())
}