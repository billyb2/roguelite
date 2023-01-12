use macroquad::prelude::*;
use once_cell::sync::Lazy;
use std::fmt::Display;

use crate::attacks::{Attack, BlindingLight, MagicMissile, Slash, Stab, ThrownKnife};
use crate::draw::{load_my_image, Drawable, Textures};
use crate::enchantments::{Enchantable, Enchantment, EnchantmentKind};
use crate::map::{Floor, FloorInfo, TILE_SIZE};
use crate::math::{easy_polygon, AsPolygon, Polygon};
use crate::player::{Player, Spell};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PotionType {
	Regeneration,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ItemType {
	ShortSword,
	WizardsDagger,
	WizardGlove,
	ThrowingKnife,
	Gold(u32),
	Potion(PotionType),
}

pub enum ItemPos {
	TilePos(IVec2),
	InventoryPos(u8),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ItemInfo {
	cursed: bool,
	pub item_type: ItemType,
	// If there is no pos, it's in the player's inventory
	tile_pos: Option<IVec2>,
	pub stack_count: Option<u8>,
}

impl ItemInfo {
	// Creates a default item
	pub fn new(item_type: ItemType, tile_pos: Option<IVec2>) -> Self {
		Self {
			cursed: false,
			item_type,
			tile_pos,
			stack_count: match item_type {
				ItemType::ThrowingKnife => Some(1),
				ItemType::Potion(_) => Some(1),
				_ => None,
			},
		}
	}

	pub fn description(&self) -> String {
		let mut description = match self.item_type {
			ItemType::WizardGlove => "A glove wielded by mighty sorcerers. Thiey alow magic users to directly tough the energy around them and manipulate it to their will.",
			ItemType::ShortSword => "A sturdy short sword, passed down from many generations.",
			ItemType::WizardsDagger => "A dagger engraved with mystical runes",
			ItemType::ThrowingKnife => "A small but very sharp knife",
			ItemType::Gold(_) => "Gold! Currency! Can be used at shops to purchase items",
			ItemType::Potion(potion_kind) => match potion_kind {
				PotionType::Regeneration => "Helps the body to recover from damage",
			},
		}.to_string();

		if self.cursed {
			description.push_str("\nMalevolant energy slithers from it.");
		}

		description
	}

	pub fn tile_pos(&self) -> Option<IVec2> { self.tile_pos }
}

impl Display for ItemInfo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&match self.item_type {
			ItemType::ShortSword => "Short Sword".to_string(),
			ItemType::WizardGlove => "Wizard's Glove".to_string(),
			ItemType::WizardsDagger => "Wizard's Dagger".to_string(),
			ItemType::ThrowingKnife => "Throwing Knife".to_string(),
			ItemType::Gold(amt) => format!("{amt} gold"),
			ItemType::Potion(potion_type) => format!(
				"Potion of {}",
				match potion_type {
					PotionType::Regeneration => "Regeneration",
				}
			),
		})
	}
}

pub fn attack_with_item(
	item: ItemInfo, player: &mut Player, index: Option<usize>, floor: &FloorInfo,
	primary_attack: bool,
) -> Option<Box<dyn Attack>> {
	match item.item_type {
		ItemType::ShortSword => Some(Slash::new(
			player,
			index,
			player.angle,
			&floor.floor,
			primary_attack,
		)),
		ItemType::WizardsDagger => Some(Stab::new(
			player,
			index,
			player.angle,
			&floor.floor,
			primary_attack,
		)),
		ItemType::WizardGlove => player.spells().get(0).copied().map(|spell| {
			let attack: Box<dyn Attack> = match spell {
				Spell::BlindingLight => {
					BlindingLight::new(player, index, player.angle, &floor.floor, primary_attack)
				},
				Spell::MagicMissile => {
					MagicMissile::new(player, index, player.angle, &floor.floor, primary_attack)
				},
			};

			attack
		}),
		ItemType::ThrowingKnife => Some(ThrownKnife::new(
			player,
			index,
			player.angle,
			&floor.floor,
			primary_attack,
		)),
		ItemType::Potion(_) => None,
		ItemType::Gold(_) => None,
	}
}

impl AsPolygon for ItemInfo {
	fn as_polygon(&self) -> Polygon {
		let half_size = self.size() * Vec2::splat(0.5);
		easy_polygon(self.pos() + half_size, half_size, 0.0)
	}
}

impl Drawable for ItemInfo {
	fn size(&self) -> Vec2 {
		match self.item_type {
			ItemType::Potion(_) => Vec2::splat(18.0),
			_ => Vec2::splat(30.0),
		}
	}

	fn pos(&self) -> Vec2 {
		(self.tile_pos.unwrap_or(IVec2::ZERO) * IVec2::splat(TILE_SIZE as i32)).as_vec2() +
			self.size() / 2.0
	}

	fn texture(&self) -> Option<Texture2D> {
		Some(load_my_image(match self.item_type {
			ItemType::Gold(_) => "gold.webp",
			ItemType::Potion(potion) => match potion {
				PotionType::Regeneration => "potion_of_regeneration.webp",
			},
			ItemType::ThrowingKnife => "throwing_knife.webp",
			_ => "gold.webp",
		}))
	}
}

type UseItemFn = Lazy<Box<dyn Fn(&ItemInfo, &mut Player, &mut Floor)>>;

pub fn use_item(item_type: &ItemType) -> Option<UseItemFn> {
	match item_type {
		ItemType::Gold(_) => None,
		ItemType::Potion(potion) => match potion {
			PotionType::Regeneration => Some(Lazy::new(|| {
				Box::new(
					|_item: &ItemInfo, player: &mut Player, _floor: &mut Floor| {
						player.apply_enchantment(Enchantment {
							kind: EnchantmentKind::Regenerating,
							strength: 1,
						})
					},
				)
			})),
		},
		ItemType::ThrowingKnife => None,
		ItemType::WizardGlove => None,
		ItemType::WizardsDagger => None,
		ItemType::ShortSword => None,
	}
}
