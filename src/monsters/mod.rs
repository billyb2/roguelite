mod small_rat;

use std::collections::HashMap;
use std::collections::HashSet;

use crate::draw::Drawable;
use crate::enchantments::Enchantable;
use crate::enchantments::Enchantment;
use crate::map::Floor;
use crate::math::{AsAABB, AxisAlignedBoundingBox};
use crate::player::DamageInfo;
use crate::player::Player;

use macroquad::prelude::*;

use rayon::prelude::*;
pub use small_rat::*;

#[derive(PartialEq, Eq, Hash)]
struct Effect {
	enchantment: Enchantment,
	frames_left: u8,
}

// All monsters are required to have a drawable AABB and be drawable
pub trait Monster: AsAABB + Drawable + Send + Enchantable {
	fn new(textures: &HashMap<String, Texture2D>, floor: &Floor) -> Self
	where
		Self: Sized;
	// Movement and damaging players are seperate so that the movement part can be
	// run in parallel
	fn movement(&mut self, players: &[Player], floor: &Floor);
	fn damage_players(&mut self, players: &mut [Player], floor: &Floor);
	fn take_damage(&mut self, damage_info: DamageInfo, floor: &Floor);
	fn living(&self) -> bool;
	/// The players to give XP to, and how much XP to give
	fn xp(&self) -> (&HashSet<usize>, u32);
	fn as_aabb_obj(&self) -> AxisAlignedBoundingBox {
		self.as_aabb()
	}
}

pub fn update_monsters(
	monsters: &mut Vec<Box<dyn Monster>>, players: &mut [Player], floor: &Floor,
) {
	monsters.par_iter_mut().for_each(|m| {
		// Only move monsters that are within a certain distance of any player
		m.update_enchantments();
		m.movement(players, floor);
	});

	monsters.retain_mut(|m| {
		m.damage_players(players, floor);
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
