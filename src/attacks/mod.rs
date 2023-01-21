mod blinding_light;
mod magic_missle;
mod slash;
mod slimeball;
mod stab;
mod throwing_knife;

use crate::draw::Drawable;
use crate::map::{Floor, FloorInfo};

use crate::math::{AsPolygon, Polygon};
use crate::player::Player;

pub use blinding_light::*;
pub use magic_missle::*;
use serde::Serialize;
pub use slash::*;
pub use slimeball::*;
pub use stab::*;
pub use throwing_knife::*;

use macroquad::prelude::*;

#[derive(Clone, Serialize)]
pub enum AttackObj {
	BlindingLight(BlindingLight),
	MagicMissile(MagicMissile),
	Slash(Slash),
	Slimeball(Slimeball),
	Stab(Stab),
	ThrowingKnife(ThrownKnife),
}

impl AttackObj {
	pub fn side_effects(&self, player: &mut Player, floor: &Floor) {
		match self {
			AttackObj::BlindingLight(obj) => obj.side_effects(player, floor),
			AttackObj::MagicMissile(obj) => obj.side_effects(player, floor),
			AttackObj::Slash(obj) => obj.side_effects(player, floor),
			AttackObj::Slimeball(obj) => obj.side_effects(player, floor),
			AttackObj::Stab(obj) => obj.side_effects(player, floor),
			AttackObj::ThrowingKnife(obj) => obj.side_effects(player, floor),
		}
	}

	pub fn mana_cost(&self) -> u16 {
		match self {
			AttackObj::BlindingLight(obj) => obj.mana_cost(),
			AttackObj::MagicMissile(obj) => obj.mana_cost(),
			AttackObj::Slash(obj) => obj.mana_cost(),
			AttackObj::Slimeball(obj) => obj.mana_cost(),
			AttackObj::Stab(obj) => obj.mana_cost(),
			AttackObj::ThrowingKnife(obj) => obj.mana_cost(),
		}
	}

	pub fn update(&mut self, floor: &mut FloorInfo, players: &mut [Player]) -> bool {
		match self {
			AttackObj::BlindingLight(obj) => obj.update(floor, players),
			AttackObj::MagicMissile(obj) => obj.update(floor, players),
			AttackObj::Slash(obj) => obj.update(floor, players),
			AttackObj::Slimeball(obj) => obj.update(floor, players),
			AttackObj::Stab(obj) => obj.update(floor, players),
			AttackObj::ThrowingKnife(obj) => obj.update(floor, players),
		}
	}

	pub fn cooldown(&self) -> u16 {
		match self {
			AttackObj::BlindingLight(obj) => obj.cooldown(),
			AttackObj::MagicMissile(obj) => obj.cooldown(),
			AttackObj::Slash(obj) => obj.cooldown(),
			AttackObj::Slimeball(obj) => obj.cooldown(),
			AttackObj::Stab(obj) => obj.cooldown(),
			AttackObj::ThrowingKnife(obj) => obj.cooldown(),
		}
	}
}

impl Drawable for AttackObj {
	fn size(&self) -> Vec2 {
		match self {
			AttackObj::BlindingLight(obj) => obj.size(),
			AttackObj::MagicMissile(obj) => obj.size(),
			AttackObj::Slash(obj) => obj.size(),
			AttackObj::Slimeball(obj) => obj.size(),
			AttackObj::Stab(obj) => obj.size(),
			AttackObj::ThrowingKnife(obj) => obj.size(),
		}
	}

	fn pos(&self) -> Vec2 {
		match self {
			AttackObj::BlindingLight(obj) => obj.pos(),
			AttackObj::MagicMissile(obj) => obj.pos(),
			AttackObj::Slash(obj) => obj.pos(),
			AttackObj::Slimeball(obj) => obj.pos(),
			AttackObj::Stab(obj) => obj.pos(),
			AttackObj::ThrowingKnife(obj) => obj.pos(),
		}
	}

	fn texture(&self) -> Option<Texture2D> {
		match self {
			AttackObj::BlindingLight(obj) => obj.texture(),
			AttackObj::MagicMissile(obj) => obj.texture(),
			AttackObj::Slash(obj) => obj.texture(),
			AttackObj::Slimeball(obj) => obj.texture(),
			AttackObj::Stab(obj) => obj.texture(),
			AttackObj::ThrowingKnife(obj) => obj.texture(),
		}
	}

	fn rotation(&self) -> f32 {
		match self {
			AttackObj::BlindingLight(obj) => obj.rotation(),
			AttackObj::MagicMissile(obj) => obj.rotation(),
			AttackObj::Slash(obj) => obj.rotation(),
			AttackObj::Slimeball(obj) => obj.rotation(),
			AttackObj::Stab(obj) => obj.rotation(),
			AttackObj::ThrowingKnife(obj) => obj.rotation(),
		}
	}

	fn flip_x(&self) -> bool {
		match self {
			AttackObj::BlindingLight(obj) => obj.flip_x(),
			AttackObj::MagicMissile(obj) => obj.flip_x(),
			AttackObj::Slash(obj) => obj.flip_x(),
			AttackObj::Slimeball(obj) => obj.flip_x(),
			AttackObj::Stab(obj) => obj.flip_x(),
			AttackObj::ThrowingKnife(obj) => obj.flip_x(),
		}
	}
}

pub trait Attack: Drawable + Send + Sync + Clone + Serialize {
	/// Just gives some information about the attack
	fn new(
		player: &dyn AsPolygon, player_index: Option<usize>, angle: f32, floor: &Floor,
		is_primary: bool,
	) -> Self;
	/// If the attack has any side effects on the user, do them here
	fn side_effects(&self, player: &mut Player, floor: &Floor);
	fn mana_cost(&self) -> u16;
	// Returns whether or not the attack should be destroyed
	fn update(&mut self, floor: &mut FloorInfo, players: &mut [Player]) -> bool;
	fn cooldown(&self) -> u16;
	fn as_polygon_optional(&self) -> Option<Polygon> { None }
}

pub fn update_attacks(players: &mut [Player], floor: &mut FloorInfo, attacks: &mut Vec<AttackObj>) {
	attacks.retain_mut(|attack| !attack.update(floor, players));
}
