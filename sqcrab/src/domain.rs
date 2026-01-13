use squirrel::err::SquirrelError;
use squirrel::vm::SquirrelVM;

pub trait DomainRegistrar {
    fn add_functions(vm: &mut SquirrelVM) -> Result<(), SquirrelError>;
}