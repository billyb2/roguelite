use serde::Serialize;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum EnchantmentKind {
	Blinded,
	Sticky,
	Regenerating,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Enchantment {
	pub kind: EnchantmentKind,
	pub strength: u8,
}

pub trait Enchantable {
	fn apply_enchantment(&mut self, enchantment: Enchantment);
	fn update_enchantments(&mut self);
}
