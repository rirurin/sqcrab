use std::error::Error;
use std::io::{Cursor, Read, Seek, SeekFrom};
use crate::from_slice;
use crate::utils::slice::FromSlice;
use crate::utils::endianness::Endianness;
use crate::utils::error::SquirrelBinaryError;
use crate::utils::reader::bytes_from_stream;

const MAGIC_FAFA: u16 = 0xfafa;
const MAGIC_SQIR: u32 = 0x53514952;

const NUT_HEADER_SIZE: usize = 0x12;

#[derive(Debug)]
#[allow(dead_code)]
pub struct NutHeader {
    sizeof_char: u32,
    sizeof_int: u32,
    sizeof_float: u32
}

impl NutHeader {
    pub fn new<E: Endianness>(stream: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
        let slice = &stream.get_ref()[stream.position() as usize..];
        if from_slice!(&slice, u16, E, 0x0) != MAGIC_FAFA { return Err(Box::new(SquirrelBinaryError::InvalidFAFAHeader)) }
        if from_slice!(&slice, u32, E, 0x2) != MAGIC_SQIR { return Err(Box::new(SquirrelBinaryError::InvalidSQIRError)) }
        stream.seek(SeekFrom::Current(NUT_HEADER_SIZE as _))?;
        Ok(Self {
            sizeof_char: from_slice!(&slice, u32, E, 0x6),
            sizeof_int: from_slice!(&slice, u32, E, 0xa),
            sizeof_float: from_slice!(&slice, u32, E, 0xe),
        })
    }
    pub fn from_generic<E: Endianness, R: Read>(stream: &mut R) -> Result<Self, Box<dyn Error>> {
        let slice = bytes_from_stream::<R, NUT_HEADER_SIZE>(stream)?;
        if from_slice!(&slice, u16, E, 0x0) != MAGIC_FAFA { return Err(Box::new(SquirrelBinaryError::InvalidFAFAHeader)) }
        if from_slice!(&slice, u32, E, 0x2) != MAGIC_SQIR { return Err(Box::new(SquirrelBinaryError::InvalidSQIRError)) }
        Ok(Self {
            sizeof_char: from_slice!(&slice, u32, E, 0x6),
            sizeof_int: from_slice!(&slice, u32, E, 0xa),
            sizeof_float: from_slice!(&slice, u32, E, 0xe),
        })
    }
}

const MAGIC_TAIL: u32 = 0x5441494c;
const NUT_TAIL_SIZE: usize = 0x4;

pub struct NutEnd;
impl NutEnd {
    pub fn new<E: Endianness>(stream: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
        let slice = &stream.get_ref()[stream.position() as usize..];
        stream.seek(SeekFrom::Current(NUT_TAIL_SIZE as _))?;
        if from_slice!(&slice, u32, E, 0x0) == MAGIC_TAIL { Ok(NutEnd) }
        else { Err(Box::new(SquirrelBinaryError::InvalidTail)) }
    }
    pub fn from_generic<E: Endianness, R: Read>(stream: &mut R) -> Result<Self, Box<dyn Error>> {
        let slice = bytes_from_stream::<R, NUT_TAIL_SIZE>(stream)?;
        if from_slice!(&slice, u32, E, 0x0) == MAGIC_TAIL { Ok(NutEnd) }
        else { Err(Box::new(SquirrelBinaryError::InvalidTail)) }
    }
}

pub(crate) const PART: u32 = 0x50415254;

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::fs::File;
    use std::io::{BufReader, Cursor, Seek, SeekFrom};
    use crate::binary::{NutEnd, NutHeader};
    use crate::utils::endianness::LittleEndian;

    #[test]
    fn read_enc0000_header_generic() -> Result<(), Box<dyn Error>> {
        let path = "E:/Metaphor/base_cpk/COMMON/battle/event/script/enc0000.cnut";
        if !std::fs::exists(path)? { return Ok(()) }
        let mut reader = BufReader::new(File::open(path)?);
        let header = NutHeader::from_generic::<LittleEndian, _>(&mut reader)?;
        assert_eq!(header.sizeof_char, 1);
        assert_eq!(header.sizeof_int, 8);
        assert_eq!(header.sizeof_float, 4);
        Ok(())
    }

    #[test]
    fn read_enc0000_header_slice() -> Result<(), Box<dyn Error>> {
        let path = "E:/Metaphor/base_cpk/COMMON/battle/event/script/enc0000.cnut";
        if !std::fs::exists(path)? { return Ok(()) }
        let bytes = std::fs::read(path)?;
        let mut bytes = Cursor::new(bytes.as_slice());
        let header = NutHeader::new::<LittleEndian>(&mut bytes)?;
        assert_eq!(header.sizeof_char, 1);
        assert_eq!(header.sizeof_int, 8);
        assert_eq!(header.sizeof_float, 4);
        Ok(())
    }

    #[test]
    fn read_enc0000_tail() -> Result<(), Box<dyn Error>> {
        let path = "E:/Metaphor/base_cpk/COMMON/battle/event/script/enc0000.cnut";
        if !std::fs::exists(path)? { return Ok(()) }
        let bytes = std::fs::read(path)?;
        let mut bytes = Cursor::new(bytes.as_slice());
        bytes.seek(SeekFrom::Start(0x148ed))?;
        let _ = NutEnd::new::<LittleEndian>(&mut bytes)?;
        Ok(())
    }
}