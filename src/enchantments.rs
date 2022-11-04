#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum EnchantmentKind {
    Blinded,

}

#[derive(PartialEq, Eq, Hash)]
pub struct Enchantment {
    pub kind: EnchantmentKind,
    pub level: u8,

}
