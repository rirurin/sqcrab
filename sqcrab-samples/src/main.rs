use std::error::Error;
use sqcrab::squirrel::squirrel;
use crate::unit::Unit;

pub mod sqcrab_domains;
pub mod unit;

fn execute() -> Result<(), Box<dyn Error>> {
    let mut script = sqcrab::SqCrab::new().build();
    script.register::<sqcrab_domains::Test>()?;
    let mut unit = Unit::default();
    println!("Start: {:?}", unit);
    let hp = squirrel!(script unit_get_hp(&unit, &Unit) -> u32)?;
    println!("HP was {}", hp);
    squirrel!(script unit_set_hp(&mut unit, &mut Unit, 75, u32))?;
    let hp = squirrel!(script unit_get_hp(&unit, &Unit) -> u32)?;
    println!("HP is now {}", hp);
    println!("End: {:?}", unit);
    Ok(())
}

fn main() {
    execute().unwrap();
}
