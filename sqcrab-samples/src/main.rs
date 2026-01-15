use std::error::Error;
use std::path::PathBuf;
use sqcrab::squirrel::squirrel;
use crate::unit::Unit;

pub mod sqcrab_domains;
pub mod unit;

fn execute() -> Result<(), Box<dyn Error>> {
    let mut unit = Unit::default();
    println!("Start: {:?}", unit);
    let mut script = sqcrab::SqCrab::<_, Unit>::new().build();
    script.register::<sqcrab_domains::Test>()?;
    let exe_dir = PathBuf::from(std::env::current_exe()?.parent().unwrap());
    script.import_text_from_file(exe_dir.join("unit.nut"))?;
    script.using_this(&mut unit, |script| {
        let hp = squirrel!(script unit_get_hp(&Unit) -> u32)?;
        println!("HP was {}", hp);
        squirrel!(script unit_set_hp(&mut Unit, 75, u32))?;
        let hp = squirrel!(script unit_get_hp(&Unit) -> u32)?;
        println!("HP is now {}", hp);
        println!("Curr: {:?}", unit);
        squirrel!(script replenish_unit())?;
        println!("After script: {:?}", unit);
        Ok(())
    })?;
    Ok(())
}

fn main() {
    execute().unwrap();
}
