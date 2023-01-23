mod slime;
mod small_rat;

use std::collections::HashSet;

use crate::attacks::AttackObj;
use crate::draw::Drawable;
use crate::enchantments::{Enchantable, Enchantment};
use crate::map::{Floor, FloorInfo};
use crate::math::{AsPolygon, Polygon};
use crate::player::{DamageInfo, Player};

use macroquad::prelude::*;

#[cfg(feature = "native")]
use rayon::prelude::*;
use serde::Serialize;
pub use slime::*;
pub use small_rat::*;

#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
struct Effect {
	enchantment: Enchantment,
	frames_left: u16,
}

#[derive(Clone, Serialize)]
pub enum MonsterObj {
	SmallRat(SmallRat),
	GreenSlime(GreenSlime),
}

impl MonsterObj {
	pub fn movement(&mut self, players: &[Player], floor: &Floor) {
		match self {
			MonsterObj::SmallRat(obj) => obj.movement(players, floor),
			MonsterObj::GreenSlime(obj) => obj.movement(players, floor),
		}
	}

	pub fn damage_players(&mut self, players: &mut [Player], floor: &Floor) {
		match self {
			MonsterObj::SmallRat(obj) => obj.damage_players(players, floor),
			MonsterObj::GreenSlime(obj) => obj.damage_players(players, floor),
		}
	}

	pub fn take_damage(&mut self, damage_info: DamageInfo, floor: &Floor) {
		match self {
			MonsterObj::SmallRat(obj) => obj.take_damage(damage_info, floor),
			MonsterObj::GreenSlime(obj) => obj.take_damage(damage_info, floor),
		}
	}

	pub fn living(&self) -> bool {
		match self {
			MonsterObj::SmallRat(obj) => obj.living(),
			MonsterObj::GreenSlime(obj) => obj.living(),
		}
	}

	pub fn xp(&self) -> (&HashSet<usize>, u32) {
		match self {
			MonsterObj::SmallRat(obj) => obj.xp(),
			MonsterObj::GreenSlime(obj) => obj.xp(),
		}
	}

	fn attack(&mut self, players: &[Player], floor: &Floor, attacks: &mut Vec<AttackObj>) {
		match self {
			MonsterObj::SmallRat(obj) => obj.attack(players, floor, attacks),
			MonsterObj::GreenSlime(obj) => obj.attack(players, floor, attacks),
		}
	}
}

impl Enchantable for MonsterObj {
	fn apply_enchantment(&mut self, enchantment: Enchantment) {
		match self {
			MonsterObj::SmallRat(obj) => obj.apply_enchantment(enchantment),
			MonsterObj::GreenSlime(obj) => obj.apply_enchantment(enchantment),
		}
	}

	fn update_enchantments(&mut self) {
		match self {
			MonsterObj::SmallRat(obj) => obj.update_enchantments(),
			MonsterObj::GreenSlime(obj) => obj.update_enchantments(),
		}
	}
}

impl Drawable for MonsterObj {
	fn size(&self) -> Vec2 {
		match self {
			MonsterObj::SmallRat(obj) => obj.size(),
			MonsterObj::GreenSlime(obj) => obj.size(),
		}
	}

	fn pos(&self) -> Vec2 {
		match self {
			MonsterObj::SmallRat(obj) => obj.pos(),
			MonsterObj::GreenSlime(obj) => obj.pos(),
		}
	}

	fn rotation(&self) -> f32 {
		match self {
			MonsterObj::SmallRat(obj) => obj.rotation(),
			MonsterObj::GreenSlime(obj) => obj.rotation(),
		}
	}

	fn texture(&self) -> Option<Texture2D> {
		match self {
			MonsterObj::SmallRat(obj) => obj.texture(),
			MonsterObj::GreenSlime(obj) => obj.texture(),
		}
	}

	fn flip_x(&self) -> bool {
		match self {
			MonsterObj::SmallRat(obj) => obj.flip_x(),
			MonsterObj::GreenSlime(obj) => obj.flip_x(),
		}
	}
}

impl AsPolygon for MonsterObj {
	fn as_polygon(&self) -> Polygon {
		match self {
			MonsterObj::SmallRat(obj) => obj.as_polygon(),
			MonsterObj::GreenSlime(obj) => obj.as_polygon(),
		}
	}
}

// All monsters are required to have a drawable AABB and be drawable
pub trait Monster: AsPolygon + Drawable + Send + Sync + Enchantable + Clone + Serialize {
	fn new(pos: Vec2) -> Self;
	// Movement and damaging players are seperate so that the movement part can be
	// run in parallel
	fn movement(&mut self, players: &[Player], floor: &Floor);
	fn attack(&mut self, _players: &[Player], _floor: &Floor, _attacks: &mut Vec<AttackObj>) {}
	fn damage_players(&mut self, players: &mut [Player], floor: &Floor);
	fn take_damage(&mut self, damage_info: DamageInfo, floor: &Floor);
	fn living(&self) -> bool;
	/// The players to give XP to, and how much XP to give
	fn xp(&self) -> (&HashSet<usize>, u32);
}

pub fn update_monsters(
	players: &mut [Player], floor_info: &mut FloorInfo, attacks: &mut Vec<AttackObj>,
) {
	#[cfg(not(feature = "native"))]
	let monsters_iter = floor_info.monsters.iter_mut();

	#[cfg(feature = "native")]
	let monsters_iter = floor_info.monsters.par_chunks_mut(4);

	monsters_iter.flatten().for_each(|m| {
		// Only move monsters that are within a certain distance of any player
		m.update_enchantments();
		m.movement(players, &floor_info.floor);
	});

	let floor = &floor_info.floor;
	let monsters = &mut floor_info.monsters;

	monsters.retain_mut(|m| {
		m.attack(players, floor, attacks);
		m.damage_players(players, &floor);
		let living = m.living();

		// If a monster dies, give all players who damaged it some XP
		if !living {
			let (indices, xp) = m.xp();

			indices.iter().copied().for_each(|i| {
				players[i].add_xp(xp);
			});
		}

		living
	});
}
