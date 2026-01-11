use std::error::Error;
use squirrel::print_cb::DebugHookType;
use squirrel::sqvm_call;
use squirrel::vm::SquirrelVM;
use squirrel_sys::bindings::root::{SQChar, SQInteger, HSQUIRRELVM};

#[test]
fn check_version() -> Result<(), Box<dyn Error>> {
    let (major, minor) = SquirrelVM::version();
    assert_eq!(major, 3);
    assert_eq!(minor, 20);
    Ok(())
}

#[test]
fn test_call_squirrel_function() -> Result<(), Box<dyn Error>> {
    let mut sqvm = SquirrelVM::new()
        .set_print_fn(|str| println!("{}", str))
        .set_error_fn(|str| println!("error: {}", str))
        .set_enable_debug_info(true)
        .set_notify_all_exceptions(true)
        .set_compile_error_cb(| desc, src, line, col| {
            println!("compile error: '{}' @ '{}', {}:{}", desc, src, line, col);
        })
        /*
        .set_debug_hook_cb(|event, source, line, function| {
            let fmt =  match event {
                DebugHookType::CallFunc => format!("debug: CALL '{}' @ {}:{}", function, source, line),
                DebugHookType::ExecLine => format!("debug: LINE '{}' @ {}:{}", function, source, line),
                DebugHookType::RetFunc => format!("debug: RET '{}' @ {}:{}", function, source, line),
                _ => "debug: received an unknown event".to_string()
            };
            println!("{}", &fmt);
        })
        */
        .build();
    let path = std::env::current_dir()?.join("tests/data/functions.nut");
    sqvm.import_text_from_file(path)?;
    assert_eq!(sqvm_call!(sqvm test1() -> u32)?, 10);
    assert_eq!(sqvm_call!(sqvm square(6, u32) -> u32)?, 36);
    assert_eq!(sqvm_call!(sqvm ack(2, u32, 1, u32) -> u32)?, 5);
    Ok(())
}