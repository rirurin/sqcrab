use std::ops::{BitAndAssign, BitOrAssign};
use bitflags::bitflags;
// use riri_mod_tools_rt::logln;
use squirrel::print_cb::DebugHookType;
use squirrel::vm::SquirrelDebugCallback;
use crate::crab::SqCrab;

bitflags! {
    #[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
    pub struct DebuggerFlags : u32 {
        const RUN_PRINT_FUNC = 1 << 0;
        const RUN_ERROR_FUNC = 1 << 1;
        const RUN_COMPILER_ERROR = 1 << 2;
        const RUN_DEBUG_HOOK = 1 << 3;
        const RUN_EXCEPTION = 1 << 4;
    }
}

impl Default for DebuggerFlags {
    fn default() -> Self {
        Self::RUN_PRINT_FUNC | Self::RUN_ERROR_FUNC | Self::RUN_COMPILER_ERROR | Self::RUN_EXCEPTION
    }
}

pub trait ScriptDebugger {
    fn print_func(&self, str: &str);
    fn error_func(&self, str: &str);
    fn on_compiler_error(&self, desc: &str, src: &str, line: i64, col: i64);
    fn on_debug(&self, event: DebugHookType, source: &str, line: i64, function: &str);
    fn get_flags(&self) -> DebuggerFlags;
    fn set_flags(&mut self, v: DebuggerFlags);
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum FunctionBreakpointData {
    OnCall,
    OnLine(i64),
    OnReturn,
    Unknown
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct FunctionBreakpoint {
    name: String,
    data: FunctionBreakpointData
}

#[derive(Debug)]
pub struct CrabDebugger {
    debugger_flags: DebuggerFlags,
    breakpoints: Vec<FunctionBreakpoint>
}

impl ScriptDebugger for CrabDebugger {
    fn print_func(&self, str: &str) {
        println!("{}", str);
    }

    fn error_func(&self, str: &str) {
        println!("{}", str);
    }

    fn on_compiler_error(&self, desc: &str, src: &str, line: i64, col: i64) {
        println!("compile error: '{}' @ '{}', {}:{}", desc, src, line, col);
    }

    fn on_debug(&self, event: DebugHookType, source: &str, line: i64, function: &str) {
        let fmt = match event {
            DebugHookType::CallFunc => format!("debug: CALL '{}' @ {}:{}", function, source, line),
            DebugHookType::ExecLine => format!("debug: LINE '{}' @ {}:{}", function, source, line),
            DebugHookType::RetFunc => format!("debug: RET '{}' @ {}:{}", function, source, line),
            _ => "debug: received an unknown event".to_string()
        };
        println!("{}", &fmt);
    }

    fn get_flags(&self) -> DebuggerFlags {
        self.debugger_flags
    }

    fn set_flags(&mut self, v: DebuggerFlags) {
        self.debugger_flags = v;
    }
}

impl CrabDebugger {
    pub fn new(debugger_flags: DebuggerFlags) -> Self {
        Self { debugger_flags, breakpoints: vec![] }
    }
}

impl BitOrAssign<DebuggerFlags> for CrabDebugger {
    fn bitor_assign(&mut self, rhs: DebuggerFlags) {
        self.debugger_flags.bitor_assign(rhs);
    }
}

impl BitAndAssign<DebuggerFlags> for CrabDebugger {
    fn bitand_assign(&mut self, rhs: DebuggerFlags) {
        self.debugger_flags.bitand_assign(rhs);
    }
}

/*
#[derive(Debug)]
pub struct DebuggerInit {
    print_cb: Option<<Self as SquirrelDebugCallback>::PrintCallback>,
    error_cb: Option<<Self as SquirrelDebugCallback>::PrintCallback>,
    compile_error_cb: Option<<Self as SquirrelDebugCallback>::CompileErrorCallback>,
    debug_hook_cb: Option<<Self as SquirrelDebugCallback>::DebugHookCallback>,
    runtime_error_cb: Option<<Self as SquirrelDebugCallback>::RuntimeErrorCallback>,
}
*/