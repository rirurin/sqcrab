use sqcrab_macro::{sqcrab, sqcrab_hint, SqObject};

#[derive(Debug, SqObject)]
pub struct Unit {
    id: u32,
    hp: u32,
    mp: u32
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            id: 1,
            hp: 50,
            mp: 40
        }
    }
}

#[sqcrab_hint]
impl Unit {
    #[sqcrab(name = "unit_get_hp", domain = "Test")]
    pub fn get_hp(&self) -> u32 { self.hp }
    #[sqcrab(name = "unit_set_hp", domain = "Test")]
    pub fn set_hp(&mut self, v: u32) { self.hp = v }
    #[sqcrab(name = "unit_get_mp", domain = "Test")]
    pub fn get_mp(&self) -> u32 { self.mp }
    #[sqcrab(name = "unit_set_mp", domain = "Test")]
    pub fn set_mp(&mut self, v: u32) { self.mp = v }
}