#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum EnchantmentKind {
	Blinded,
	Slimed,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Enchantment {
	pub kind: EnchantmentKind,
	pub strength: u16,
}

pub trait Enchantable {
	fn apply_enchantment(&mut self, enchantment: Enchantment);
	fn update_enchantments(&mut self);
}
