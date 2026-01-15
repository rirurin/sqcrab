pub mod binary;
pub mod function;
pub mod instruction;
pub mod line_info;
pub mod local_var;
pub mod object;
pub mod outer_val;

pub mod utils {
    pub mod endianness;
    pub mod error;
    pub mod reader;
    pub mod slice;
}