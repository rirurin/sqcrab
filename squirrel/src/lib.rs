pub mod err;
pub mod obj_type;
pub mod object;
pub mod print_cb;
pub mod type_cnv;
pub mod vm;

// Re-export squirrel-sys crate
pub use ::squirrel_sys;
// Recursively define self as squirrel, lets us use squirrel! macro outside of this crate
pub use crate as squirrel;