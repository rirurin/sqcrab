use squirrel_sys::bindings::root::*;
use crate::err::SquirrelError;
use crate::obj_type::SquirrelObject;
use crate::vm::SquirrelVM;

pub trait CanSquirrel where Self: Sized {
    type Into: SquirrelObject;

    const RETURNS: bool;

    fn into_squirrel(&self) -> Self::Into;

    fn from_squirrel(v: Self::Into) -> Self;

    fn push(&self, vm: &mut SquirrelVM) {
        <Self::Into>::push(&self.into_squirrel(), vm);
    }

    fn get(vm: &SquirrelVM, index: usize) -> Result<Self::Into, SquirrelError> {
        <Self::Into>::get(vm, index)
    }
}

macro_rules! auto_sq_object {
    ($ty:ident, $head:ident) => {
        impl CanSquirrel for $head {
            type Into = $ty;

            const RETURNS: bool = true;

            fn into_squirrel(&self) -> Self::Into {
                *self as _
            }

            fn from_squirrel(v: Self::Into) -> Self {
                v as _
            }
        }
    };

    ($ty:ident, $head:ident, $($tail:ident),*) => {
        auto_sq_object!($ty, $head);
        auto_sq_object!($ty, $($tail),*);
    };
}

auto_sq_object!(SQInteger, i8, u8, i16, u16, i32, u32, i64, u64);
auto_sq_object!(SQFloat, f32, f64);
// auto_sq_object!(SQBool, bool);

impl CanSquirrel for bool {
    type Into = SQBool;

    const RETURNS: bool = true;

    fn into_squirrel(&self) -> Self::Into {
        *self as _
    }

    fn from_squirrel(v: Self::Into) -> Self {
        v != 0
    }
}

impl CanSquirrel for () {
    type Into = ();

    const RETURNS: bool = false;

    fn into_squirrel(&self) -> Self::Into {
        *self
    }

    fn from_squirrel(v: Self::Into) -> Self {
        v
    }
}