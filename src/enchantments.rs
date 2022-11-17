#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum EnchantmentKind {
	Blinded,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Enchantment {
	pub kind: EnchantmentKind,
	pub level: u8,
}

pub trait Enchantable {
	fn apply_enchantment(&mut self, enchantment: Enchantment);
	fn update_enchantments(&mut self);
}
