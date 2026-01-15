use std::io::{Cursor, Read, Seek, SeekFrom};
use std::mem::MaybeUninit;
use crate::utils::error::SquirrelBinaryError;

pub(crate) fn bytes_from_stream<R: Read, const N: usize>(stream: &mut R) -> std::io::Result<[u8; N]> {
    let mut header: MaybeUninit<[u8; N]> = MaybeUninit::uninit();
    unsafe { stream.read_exact(header.assume_init_mut())? };
    Ok(unsafe { header.assume_init() })
}

pub(crate) fn try_advance(stream: &mut Cursor<&[u8]>, v: i64) -> Result<(), SquirrelBinaryError> {
    stream.seek(SeekFrom::Current(v))
        .map_err(|e| SquirrelBinaryError::IoError(e))?;
    Ok(())
}