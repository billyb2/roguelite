use macroquad::prelude::*;
use std::{collections::HashMap, fmt::Display};

use crate::{
	attacks::{Attack, BlindingLight, MagicMissile, Slash},
	draw::Drawable,
	map::{FloorInfo, TILE_SIZE},
	math::{AsAABB, AxisAlignedBoundingBox},
	player::{Player, Spell},
	TEXTURES,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemType {
	ShortSword,
	WizardGlove,
	Gold(u32),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ItemInfo {
	cursed: bool,
	pub item_type: ItemType,
	// If there is no pos, it's in the player's inventory
	tile_pos: Option<IVec2>,
}

impl ItemInfo {
	// Creates a default item
	pub fn new(item_type: ItemType, tile_pos: Option<IVec2>) -> Self {
		Self {
			cursed: false,
			item_type,
			tile_pos,
		}
	}

	pub fn description(&self) -> String {
		let mut description = match self.item_type {
			ItemType::WizardGlove => "A glove wielded by mighty sorcerers. Thiey alow magic users to directly tough the energy around them and manipulate it to their will.",
			ItemType::ShortSword => "A sturdy short sword, passed down from many generations.",
			ItemType::Gold(_) => "Gold! Currency! Can be used at shops to purchase items",
		}.to_string();

		if self.cursed {
			description.push_str("\nMalevolant energy slithers from it.");
		}

		description
	}
}

impl Display for ItemInfo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&match self.item_type {
			ItemType::ShortSword => "Short Sword".to_string(),
			ItemType::WizardGlove => "Wizard's Glove".to_string(),
			ItemType::Gold(amt) => format!("{amt} gold"),
		})
	}
}

pub fn attack_with_item(
	item: ItemInfo, player: &mut Player, textures: &HashMap<String, Texture2D>, floor: &FloorInfo,
	primary_attack: bool,
) -> Option<Box<dyn Attack>> {
	match item.item_type {
		ItemType::ShortSword => Some(Slash::new(
			player,
			player.angle,
			textures,
			floor,
			primary_attack,
		)),
		ItemType::WizardGlove => player.spells().get(0).copied().map(|spell| {
			let attack: Box<dyn Attack> = match spell {
				Spell::BlindingLight => {
					BlindingLight::new(player, player.angle, textures, floor, primary_attack)
				},
				Spell::MagicMissile => {
					MagicMissile::new(player, player.angle, textures, floor, primary_attack)
				},
			};

			attack
		}),
		ItemType::Gold(_) => None,
	}
}

impl AsAABB for ItemInfo {
	fn as_aabb(&self) -> AxisAlignedBoundingBox {
		AxisAlignedBoundingBox {
			pos: self.pos(),
			size: Vec2::splat(32.0),
		}
	}
}

impl Drawable for ItemInfo {
	fn size(&self) -> Vec2 {
		Vec2::splat(32.0)
	}

	fn pos(&self) -> Vec2 {
		(self.tile_pos.unwrap_or(IVec2::ZERO) * IVec2::splat(TILE_SIZE as i32)).as_vec2()
			+ self.size() / 2.0
	}

	fn texture(&self) -> Option<Texture2D> {
		Some(*TEXTURES.get("gold.webp").unwrap())
	}
}
