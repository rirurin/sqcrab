use std::error::Error;
use std::io::{Cursor, Seek, SeekFrom};
use crate::from_slice;
use crate::instruction::Instruction;
use crate::line_info::LineInfo;
use crate::local_var::LocalVar;
use crate::object::BinObject;
use crate::outer_val::OuterValue;
use crate::utils::slice::FromSlice;
use crate::utils::endianness::Endianness;

macro_rules! part {
    ($s:ident) => {
        let slice = &$s.get_ref()[$s.position() as usize..];
        if from_slice!(slice, u32, crate::utils::endianness::LittleEndian, 0x0) != crate::binary::PART {
            return Err(Box::new(crate::utils::error::SquirrelBinaryError::ExpectedPart));
        }
        $s.seek(SeekFrom::Current(4))?;
    }
}

#[derive(Debug)]
pub struct NutFunction {
    source_name: BinObject,
    name: BinObject,
    literals: Vec<BinObject>,
    parameters: Vec<BinObject>,
    outer_values: Vec<OuterValue>,
    local_var_infos: Vec<LocalVar>,
    line_infos: Vec<LineInfo>,
    default_params: Vec<i64>,
    instructions: Vec<Instruction>,
    functions: Vec<NutFunction>,
    stack_size: i64,
    is_generator: bool,
    var_params: i64
}

impl NutFunction {
    pub fn get_source_name(&self) -> &str {
        (&self.source_name).try_into().unwrap()
    }
    pub fn get_name(&self) -> &str {
        (&self.name).try_into().unwrap()
    }
    pub fn get_literals(&self) -> &[BinObject] {
        &self.literals
    }
    pub fn get_parameters(&self) -> &[BinObject] {
        &self.parameters
    }
    pub fn get_outer_values(&self) -> &[OuterValue] {
        &self.outer_values
    }
    pub fn get_local_var_infos(&self) -> &[LocalVar] {
        &self.local_var_infos
    }
    pub fn get_line_infos(&self) -> &[LineInfo] {
        &self.line_infos
    }
    pub fn get_default_params(&self) -> &[i64] {
        &self.default_params
    }
    pub fn get_instructions(&self) -> &[Instruction] {
        &self.instructions
    }
    pub fn get_inner_functions(&self) -> &[NutFunction] {
        &self.functions
    }
    pub fn get_stack_size(&self) -> i64 {
        self.stack_size
    }
    pub fn check_generator(&self) -> bool {
        self.is_generator
    }
    pub fn get_var_params(&self) -> i64 {
        self.var_params
    }

    pub fn new<E: Endianness>(stream: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
        part!(stream);
        let source_name = BinObject::new::<E>(stream)?;
        let _: &str = (&source_name).try_into()?;
        let name = BinObject::new::<E>(stream)?;
        let name_str: &str = (&name).try_into()?;
        part!(stream);
        let slice = &stream.get_ref()[stream.position() as usize..];
        let n_literals = from_slice!(slice, u64, E, 0x0);
        let n_parameters = from_slice!(slice, u64, E, 0x8);
        let n_outer_values = from_slice!(slice, u64, E, 0x10);
        let n_local_var_infos = from_slice!(slice, u64, E, 0x18);
        let n_line_infos = from_slice!(slice, u64, E, 0x20);
        let n_default_params = from_slice!(slice, u64, E, 0x28);
        let n_instructions = from_slice!(slice, u64, E, 0x30);
        let n_functions = from_slice!(slice, u64, E, 0x38);
        stream.seek(SeekFrom::Current(0x40))?;
        // Literals
        let mut literals = Vec::with_capacity(n_literals as _);
        part!(stream);
        for _ in 0..n_literals {
            literals.push(BinObject::new::<E>(stream)?);
        }
        let mut parameters = Vec::with_capacity(n_parameters as _);
        part!(stream);
        for _ in 0..n_parameters {
            parameters.push(BinObject::new::<E>(stream)?);
        }
        let mut outer_values = Vec::with_capacity(n_outer_values as _);
        part!(stream);
        for _ in 0..n_outer_values {
            outer_values.push(OuterValue::new::<E>(stream)?);
        }
        let mut local_var_infos = Vec::with_capacity(n_local_var_infos as _);
        part!(stream);
        for _ in 0..n_local_var_infos {
            local_var_infos.push(LocalVar::new::<E>(stream)?);
        }
        let mut line_infos = Vec::with_capacity(n_line_infos as _);
        part!(stream);
        for _ in 0..n_line_infos {
            line_infos.push(LineInfo::new::<E>(stream)?);
        }
        let mut default_params = Vec::with_capacity(n_default_params as _);
        part!(stream);
        let slice = &stream.get_ref()[stream.position() as usize..];
        for i in 0..n_default_params {
            default_params.push(from_slice!(slice, i64, E, 8 * i as usize));
        }
        stream.seek(SeekFrom::Current((8 * n_default_params) as _))?;
        let mut instructions = Vec::with_capacity(n_instructions as _);
        part!(stream);
        for _ in 0..n_instructions {
            instructions.push(Instruction::new::<E>(stream)?);
        }
        let mut functions = Vec::with_capacity(n_functions as _);
        part!(stream);
        for _ in 0..n_functions {
            functions.push(NutFunction::new::<E>(stream)?);
        }
        let slice = &stream.get_ref()[stream.position() as usize..];
        let stack_size = from_slice!(slice, i64, E, 0x0);
        let is_generator = slice[8] != 0;
        let var_params = from_slice!(slice, i64, E, 0x9);
        stream.seek(SeekFrom::Current(0x11))?;
        Ok(Self {
            source_name, name, literals, parameters, outer_values, local_var_infos, line_infos,
            default_params, instructions, functions, stack_size, is_generator, var_params
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::io::{Cursor, Seek, SeekFrom};
    use std::path::Path;
    use crate::function::NutFunction;
    use crate::utils::endianness::LittleEndian;

    #[test]
    fn parse_enc0000_functions() -> Result<(), Box<dyn Error>> {
        let path = "E:/Metaphor/base_cpk/COMMON/battle/event/script/enc0000.cnut";
        if !std::fs::exists(path)? { return Ok(()) }
        let bytes = std::fs::read(path)?;
        let mut bytes = Cursor::new(bytes.as_slice());
        bytes.seek(SeekFrom::Start(0x12))?;
        let _ = NutFunction::new::<LittleEndian>(&mut bytes)?;
        Ok(())
    }

    #[test]
    fn parse_00_follower_status_functions() -> Result<(), Box<dyn Error>> {
        let path = "E:/Metaphor/base_cpk/COMMON/field/source/script/npc/00_Follower_Status.cnut";
        if !std::fs::exists(path)? { return Ok(()) }
        let bytes = std::fs::read(path)?;
        let mut bytes = Cursor::new(bytes.as_slice());
        bytes.seek(SeekFrom::Start(0x12))?;
        let _ = NutFunction::new::<LittleEndian>(&mut bytes)?;
        Ok(())
    }

    fn read_directory_recursive(path: &Path) -> Result<(), Box<dyn Error>> {
        for entry in std::fs::read_dir(path)?.filter_map(|e| e.ok()) {
            if entry.file_type()?.is_dir() {
                read_directory_recursive(entry.path().as_path())?;
            } else if let Some(ext) = entry.path().extension() {
                if ext.to_str().unwrap() == "cnut" {
                    let bytes = std::fs::read(entry.path())?;
                    let mut bytes = Cursor::new(bytes.as_slice());
                    bytes.seek(SeekFrom::Start(0x12))?;
                    let _ = NutFunction::new::<LittleEndian>(&mut bytes)?;
                }
            }
        }
        Ok(())
    }

    #[test]
    #[ignore = "Takes a while"]
    fn parse_all_metaphor_scripts() -> Result<(), Box<dyn Error>> {
        let path = "E:/Metaphor/base_cpk/";
        if !std::fs::exists(path)? { return Ok(()) }
        read_directory_recursive(Path::new(path))?;
        Ok(())
    }
}