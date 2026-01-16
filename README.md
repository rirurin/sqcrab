# sqcrab

A Rust crate for binding Rust functions, methods and structs to the [Squirrel language](http://squirrel-lang.org/) and Rust bindings for Squirrel.

The crate specifically targets Squirrel 3.2 with SQ64 set to true (this makes `SQInteger` an i64).

## Features

- Expose functions and methods to sqcrab using the `#[sqcrab]` attribute to allow them to get bound to a Squirrel virtual machine
- A dedicated crate `sqcrab-builder` for generating a binding file given the path to the user's crate (Example build script shown in `sqcrab-samples`).
- Initialize and execute code in a Squirrel VM by importing native Rust functions or from a Squirrel file in source code or bytecode form
- Call Squirrel functions from Rust using the `squirrel!` macro
- Parse compiled Squirrel scripts (.cnut)

**This crate is still under development and is missing some features that I want to implement**

## Usage

### Creating a Virtual Machine

A squirrel virtual machine can be defined either using `squirrel::SquirrelVM` or `sqcrab::SqCrab` (Sqcrab provides some extra functionality such as
defining a scope for a foreign pointer or defining a custom debugging trait).

**SquirrelVM example:**
```rust
let mut sqvm = SquirrelVM::new()
    // Define callbacks - when calling SquirrelVM::new(), this will use the function signatures defined in SquirrelDebugCallbackBasic
    .callbacks(|c| {
        c.set_print_cb(|str| println!("{}", str));
        c.set_error_cb(|str| println!("error: {}", str));
        c.set_compile_error_cb(| desc, src, line, col| println!("compile error: '{}' @ '{}', {}:{}", desc, src, line, col));
        c.set_debug_hook_cb(|event, source, line, function| {
            let fmt =  match event {
                DebugHookType::CallFunc => format!("debug: CALL '{}' @ {}:{}", function, source, line),
                DebugHookType::ExecLine => format!("debug: LINE '{}' @ {}:{}", function, source, line),
                DebugHookType::RetFunc => format!("debug: RET '{}' @ {}:{}", function, source, line),
                _ => "debug: received an unknown event".to_string()
            };
            println!("{}", &fmt);
        });
    })
    // will include debug info when compiling
    .set_enable_debug_info(true)
    .set_notify_all_exceptions(true)
    .build();
```
**Sqcrab example:**
```rust
// Create a SqCrab instance with default settings, which includes setting up the debugger callbacks
let mut script = sqcrab::SqCrab::<_, Unit>::new().build();
```

### Importing Functions

Functions from Squirrel files can be imported using one of the following functions depending on if it's from source code or compiled into bytecode:

| | From Path | From Buffer |
| - | - | - |
| Squirrel source (.nut) | `import_text_from_file` | `import_text_from_str`
| Squirrel bytecode (.cnut) | `import_binary_from_file` | `import_binary_from_slice`

Given a squirrel script that contains the following,
```js
function square(n) {
    return n * n;
}
```

```rust
let path = std::env::current_dir()?.join("tests/data/functions.nut");
sqvm.import_text_from_file(path)?;
```

will add `square` into `sqvm`'s root table, which can be called by other squirrel scripts or from Rust.

To add a native function into the VM, `add_function` is used:

```rust
sqvm.add_function("square", |vm| {
    // get the first parameter

    // in functions with multiple parameters, get should be called using the inverse index
    // This is due to the order that params are added onto the VM's stack
    // (e.g for a 3 param function, the first param will be vm.get::<type>(3).unwrap())
    let p = vm.get::<u32>(1).unwrap();
    vm.push::<u32>(&(p * p));
    // the callback should return 1 if it returns a value, which we've pushed onto the stack.
    // if it doesn't, return 0
    1
})?;
```

### `squirrel!` macro

Squirrel functions can be invoked from Rust code using `squirrel!`. This abstracts all the low level operations required to set up function
invocation into a macro with syntax similar to a Rust function call.

`squirrel!` uses the format `[vm_name] [func_name]([value0], [type0], [value1], [type1] ...) -> [ret_type]`

As an example, using the square function above to get the square of 6 can be called with the following:

```rust
let result = squirrel!(sqvm square(6, u32) -> u32)?;
```

For functions that return a unit type, the return value can be omitted like in regular Rust:

```rust
squirrel!(sqvm do_action(10, u32))?;
```

Any type that implements `CanSquirrel` can be passed into these parameters, including user types if implemented:

```rust
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

// ...

squirrel!(sqvm get_hp(&unit, &TestUnit) -> u32)?;
```

Finally, when using Sqcrab, the `using_this` function can be used to set the "foreign pointer" value in a specific scope to a given value.
This allows us to omit the first parameter's value for method calls since it's implicitly defined as the value of the foreign pointer:

```rust
impl Unit {
    pub fn get_hp(&self) -> u32 { self.hp }
}

// ...

script.using_this(&mut unit, |script| {
    let hp = squirrel!(script unit_get_hp(&Unit) -> u32)?;
    println!("HP was {}", hp);
});
```

### Sqcrab Function Binding

Sqcrab contains some tools for automatically creating binds between Rust functions/structures and Squirrel functions.

User-defined structs can automatically implement `CanSquirrel` on it's reference types usign the `SqObject` macro in `sqcrab_macro`:

```rust
use sqcrab_macro::SqObject;

#[derive(Debug, SqObject)]
pub struct Unit {
    id: u32,
    hp: u32,
    mp: u32
}
```

which will provide a definition like the one shown above for `TestUnit`.

Struct methods can also be marked for function binding using the `sqcrab` attribute, and can optionally include defining a custom name 
to avoid naming conflicts and a "domain" that the function is registered under.

```rust

// sqcrab_hint is another attribute marked on impl blocks to tell the build script reading this file that sqcrab functions exist here

#[sqcrab_hint]
impl Unit {
    #[sqcrab(name = "unit_get_hp", domain = "Test")]
    pub fn get_hp(&self) -> u32 { self.hp }
    #[sqcrab(name = "unit_set_hp", domain = "Test")]
    pub fn set_hp(&mut self, v: u32) { self.hp = v }
    #[sqcrab(name = "unit_get_mp", domain = "Test")]
    pub fn get_mp(&self) -> u32 { self.mp }
    #[sqcrab(name = "unit_set_mp", domain = "Test")]
    pub fn set_mp(&mut self, v: u32) { self.mp = v }
}
```

The binding process is handled by creating a build script in your crate, importing `sqcrab_builder` as a build dependency and using it's
`build_domain_initialization` method:

```rust
fn main() {
    let root_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    sqcrab_builder::domain::build_domain_initalization(root_dir.as_path()).unwrap();
}
```

`build_domain_initialization` scans through all source files to check for sqcrab attributes and then 
construct a file (in `sqcrab_domains.rs` by default) which calls `add_function` for every marked
function.

The domain defines which group the function is part of. This allows your Sqcrab VM to import a specific set of functions using the 
`register` method:

```rust
let mut script = sqcrab::SqCrab::<_, Unit>::new().build();
script.register::<sqcrab_domains::Test>()?;
```

Optionally, a file called `sqcrab.toml` can be placed adjacent to your crate's `Cargo.toml` to change how the build script operates.
The following keys are currently supported:

```toml
# relative to src
include = [ "list" "of" "files/to/include" ]
# if omitted, this is set to sqcrab_domains
output = "sqcrab_bind_name"
```

### Parsing Compiled Squirrel Scripts

`squirrel-cnut` is available if your program requires parsing Squirrel bytecode (.cnut) to extract certain information from it. \
One example that I'm personally using is to check if all the functions referenced in a script are defined in the VM ahead of time
to avoid issues when executing the script.

## Sample Program

A small sample program is available in `sqcrab-samples` which demonstrates automatically generating bindings from a `Unit` struct into a script domain,
importing the functions from that domain into a SqCrab virtual machine, then calling a squirrel script which calls those functions.

## Credits and Resources

- **albertodemichelis** ([Github](https://github.com/albertodemichelis), [Twitter](https://x.com/squirrellang)) - [Squirrel Language](https://github.com/albertodemichelis/squirrel)
- **Sqrat Developers** (atai, tojiro, wizzard97) - [Squirrel binding library for C++](https://github.com/hakase-labs/sqrat)