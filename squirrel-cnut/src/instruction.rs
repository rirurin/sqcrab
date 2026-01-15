use std::error::Error;
use std::io::{Cursor, Seek, SeekFrom};
use crate::from_slice;
use crate::utils::endianness::{Endianness, LittleEndian};
use crate::utils::error::SquirrelBinaryError;
use crate::utils::slice::FromSlice;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum BitWiseOP {
    BW_AND = 0,
    BW_OR = 2,
    BW_XOR = 3,
    BW_SHIFTL = 4,
    BW_SHIFTR = 5,
    BW_USHIFTR = 6
}

impl TryFrom<u8> for BitWiseOP {
    type Error = SquirrelBinaryError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= Self::BW_USHIFTR as u8 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(SquirrelBinaryError::InvalidBitwiseOp(value))
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum CmpOP {
    CMP_G = 0,
    CMP_GE = 2,
    CMP_L = 3,
    CMP_LE = 4,
    CMP_3W = 5
}

impl TryFrom<u8> for CmpOP {
    type Error = SquirrelBinaryError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= Self::CMP_3W as u8 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(SquirrelBinaryError::InvalidCmpOp(value))
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum NewObjectType {
    NOT_TABLE = 0,
    NOT_ARRAY = 1,
    NOT_CLASS = 2
}

impl TryFrom<u8> for NewObjectType {
    type Error = SquirrelBinaryError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= Self::NOT_CLASS as u8 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(SquirrelBinaryError::InvalidNewObjectType(value))
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum AppendArrayType {
    AAT_STACK = 0,
    AAT_LITERAL = 1,
    AAT_INT = 2,
    AAT_FLOAT = 3,
    AAT_BOOL = 4
}

impl TryFrom<u8> for AppendArrayType {
    type Error = SquirrelBinaryError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= Self::AAT_BOOL as u8 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(SquirrelBinaryError::InvalidAppendArrayType(value))
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum SQOpcode
{
    _OP_LINE=               0x00,
    _OP_LOAD=               0x01,
    _OP_LOADINT=            0x02,
    _OP_LOADFLOAT=          0x03,
    _OP_DLOAD=              0x04,
    _OP_TAILCALL=           0x05,
    _OP_CALL=               0x06,
    _OP_PREPCALL=           0x07,
    _OP_PREPCALLK=          0x08,
    _OP_GETK=               0x09,
    _OP_MOVE=               0x0A,
    _OP_NEWSLOT=            0x0B,
    _OP_DELETE=             0x0C,
    _OP_SET=                0x0D,
    _OP_GET=                0x0E,
    _OP_EQ=                 0x0F,
    _OP_NE=                 0x10,
    _OP_ADD=                0x11,
    _OP_SUB=                0x12,
    _OP_MUL=                0x13,
    _OP_DIV=                0x14,
    _OP_MOD=                0x15,
    _OP_BITW=               0x16,
    _OP_RETURN=             0x17,
    _OP_LOADNULLS=          0x18,
    _OP_LOADROOT=           0x19,
    _OP_LOADBOOL=           0x1A,
    _OP_DMOVE=              0x1B,
    _OP_JMP=                0x1C,
    _OP_JCMP=               0x1D,
    _OP_JZ=                 0x1E,
    _OP_SETOUTER=           0x1F,
    _OP_GETOUTER=           0x20,
    _OP_NEWOBJ=             0x21,
    _OP_APPENDARRAY=        0x22,
    _OP_COMPARITH=          0x23,
    _OP_INC=                0x24,
    _OP_INCL=               0x25,
    _OP_PINC=               0x26,
    _OP_PINCL=              0x27,
    _OP_CMP=                0x28,
    _OP_EXISTS=             0x29,
    _OP_INSTANCEOF=         0x2A,
    _OP_AND=                0x2B,
    _OP_OR=                 0x2C,
    _OP_NEG=                0x2D,
    _OP_NOT=                0x2E,
    _OP_BWNOT=              0x2F,
    _OP_CLOSURE=            0x30,
    _OP_YIELD=              0x31,
    _OP_RESUME=             0x32,
    _OP_FOREACH=            0x33,
    _OP_POSTFOREACH=        0x34,
    _OP_CLONE=              0x35,
    _OP_TYPEOF=             0x36,
    _OP_PUSHTRAP=           0x37,
    _OP_POPTRAP=            0x38,
    _OP_THROW=              0x39,
    _OP_NEWSLOTA=           0x3A,
    _OP_GETBASE=            0x3B,
    _OP_CLOSE=              0x3C
}

impl TryFrom<u8> for SQOpcode {
    type Error = SquirrelBinaryError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= Self::_OP_CLOSE as u8 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(SquirrelBinaryError::InvalidOpcode(value))
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Instruction {
    arg1: [u8; 4],
    op: SQOpcode,
    arg0: u8,
    arg2: u8,
    arg3: u8
}

impl Instruction {
    pub fn new<E: Endianness>(stream: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
        let slice = &stream.get_ref()[stream.position() as usize..];
        let arg1 = <[u8; 4]>::try_from(&slice[..4])?;
        let op = slice[4].try_into()?;
        let arg0 = slice[5];
        let arg2 = slice[6];
        let arg3 = slice[7];
        stream.seek(SeekFrom::Current(8))?;
        Ok(Self {
            arg1, op, arg0, arg2, arg3
        })
    }

    pub fn get_opcode(&self) -> SQOpcode {
        self.op
    }

    pub fn arg_as<T: IntoInstructionArg>(&self) -> T {
        T::into(&self.arg1)
    }

    // arg0: target on the stack
    pub fn get_arg0(&self) -> u8 { self.arg0 }
    pub fn get_arg2(&self) -> u8 { self.arg2 }
    pub fn get_arg3(&self) -> u8 { self.arg3 }
}

pub trait IntoInstructionArg where Self: Sized {
    fn into(v: &[u8]) -> Self;
}

impl IntoInstructionArg for u32 {
    fn into(v: &[u8]) -> Self {
        from_slice!(v, u32, LittleEndian, 0)
    }
}

impl IntoInstructionArg for f32 {
    fn into(v: &[u8]) -> Self {
        from_slice!(v, f32, LittleEndian, 0)
    }
}