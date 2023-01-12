mod slime;
mod small_rat;

use std::collections::HashSet;

use crate::attacks::Attack;
use crate::draw::{Drawable, Textures};
use crate::enchantments::{Enchantable, Enchantment};
use crate::map::{Floor, FloorInfo};
use crate::math::AsPolygon;
use crate::player::{DamageInfo, Player};

use macroquad::prelude::*;

#[cfg(feature = "native")]
use rayon::prelude::*;
pub use slime::*;
pub use small_rat::*;

#[derive(PartialEq, Eq, Hash)]
struct Effect {
	enchantment: Enchantment,
	frames_left: u16,
}

// All monsters are required to have a drawable AABB and be drawable
pub trait Monster: AsPolygon + Drawable + Send + Sync + Enchantable {
	fn new(pos: Vec2) -> Box<dyn Monster>
	where
		Self: Sized;
	// Movement and damaging players are seperate so that the movement part can be
	// run in parallel
	fn movement(&mut self, players: &[Player], floor: &Floor);
	fn attack(&mut self, _players: &[Player], _floor: &Floor, _attacks: &mut Vec<Box<dyn Attack>>) {
	}
	fn damage_players(&mut self, players: &mut [Player], floor: &Floor);
	fn take_damage(&mut self, damage_info: DamageInfo, floor: &Floor);
	fn living(&self) -> bool;
	/// The players to give XP to, and how much XP to give
	fn xp(&self) -> (&HashSet<usize>, u32);
}

pub fn update_monsters(
	players: &mut [Player], floor_info: &mut FloorInfo, attacks: &mut Vec<Box<dyn Attack>>,
) {
	#[cfg(not(feature = "native"))]
	let monsters_iter = floor_info.monsters.iter_mut();

	#[cfg(feature = "native")]
	let monsters_iter = floor_info.monsters.par_iter_mut();

	monsters_iter.for_each(|m| {
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
