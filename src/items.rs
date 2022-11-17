use macroquad::prelude::*;
use std::{collections::HashMap, fmt::Display};

use crate::{
	attacks::{Attack, BlindingLight, MagicMissile, Slash},
	map::Floor,
	player::{Player, Spell},
};

#[derive(Copy, Clone)]
pub enum ItemType {
	ShortSword,
	WizardGlove,
	Gold(u32),
}

#[derive(Copy, Clone)]
pub struct ItemInfo {
	cursed: bool,
	item_type: ItemType,
}

impl ItemInfo {
	// Creates a default item
	pub fn new(item_type: ItemType) -> Self {
		Self {
			cursed: false,
			item_type,
		}
	}

	pub fn description(&self) -> String {
		let mut description = match self.item_type {
			ItemType::WizardGlove => "The glove wielded by mighty sorcers. Thiey alow magic users to directly tough the energy around them and manipulate it to their will.",
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
	item: ItemInfo, player: &mut Player, textures: &HashMap<String, Texture2D>, floor: &Floor,
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
