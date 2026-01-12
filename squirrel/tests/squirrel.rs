use std::error::Error;
use squirrel::obj_type::UserPointer;
use squirrel::print_cb::DebugHookType;
use squirrel::squirrel;
use squirrel::type_cnv::CanSquirrel;
use squirrel::vm::SquirrelVM;
use squirrel_sys::bindings::root::sq_gettype;

#[test]
fn check_version() -> Result<(), Box<dyn Error>> {
    let (major, minor) = SquirrelVM::version();
    assert_eq!(major, 3);
    assert_eq!(minor, 20);
    Ok(())
}

#[test]
fn call_squirrel_function() -> Result<(), Box<dyn Error>> {
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
    assert_eq!(squirrel!(sqvm test1() -> u32)?, 10);
    assert_eq!(squirrel!(sqvm square(6, u32) -> u32)?, 36);
    assert_eq!(squirrel!(sqvm ack(2, u32, 1, u32) -> u32)?, 5);
    Ok(())
}

#[test]
fn call_native_function_with_object() -> Result<(), Box<dyn Error>> {
    let mut sqvm = SquirrelVM::new()
        .set_print_fn(|str| println!("{}", str))
        .set_error_fn(|str| println!("error: {}", str))
        .set_enable_debug_info(true)
        .set_notify_all_exceptions(true)
        .set_compile_error_cb(| desc, src, line, col| {
            println!("compile error: '{}' @ '{}', {}:{}", desc, src, line, col);
        })
        .build();
    sqvm.add_function("square", |vm| {
        let p = vm.get::<u32>(1).unwrap();
        vm.push::<u32>(&(p * p));
        1
    })?;
    assert_eq!(squirrel!(sqvm square(6, u32) -> u32)?, 36);
    Ok(())
}

#[derive(Debug)]
struct TestUnit {
    id: u32,
    hp: u32,
    mp: u32
}

impl Default for TestUnit {
    fn default() -> Self {
        Self {
            id: 1,
            hp: 50,
            mp: 30
        }
    }
}

impl<'a> CanSquirrel for &'a TestUnit {
    type Into = UserPointer<Self>;
    const RETURNS: bool = true;

    fn into_squirrel(&self) -> Self::Into {
        UserPointer::<Self>::new(self)
    }

    fn from_squirrel(v: Self::Into) -> Self {
        unsafe { *v.as_ptr() }
    }
}

impl<'a> CanSquirrel for &'a mut TestUnit {
    type Into = UserPointer<Self>;
    const RETURNS: bool = true;

    fn into_squirrel(&self) -> Self::Into {
        UserPointer::<Self>::new(self)
    }

    fn from_squirrel(v: Self::Into) -> Self {
        unsafe { *v.as_ptr() }
    }
}

#[test]
fn call_native_method_with_object() -> Result<(), Box<dyn Error>> {
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
    let mut unit = TestUnit::default();
    sqvm.add_function("get_hp", |vm| {
        let p = vm.get::<&TestUnit>(1).unwrap();
        vm.push::<u32>(&p.hp);
        1

    })?;
    sqvm.add_function("add_hp", |vm| {
        let p = vm.get::<&mut TestUnit>(2).unwrap();
        p.hp += vm.get::<u32>(1).unwrap();
        0
    })?;
    assert_eq!(squirrel!(sqvm get_hp(&unit, &TestUnit) -> u32)?, 50);
    squirrel!(sqvm add_hp(&mut unit, &mut TestUnit, 10, u32))?;
    assert_eq!(squirrel!(sqvm get_hp(&unit, &TestUnit) -> u32)?, 60);
    Ok(())
}