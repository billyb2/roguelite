mod blinding_light;
mod magic_missle;
mod slash;

use std::collections::HashMap;

use crate::draw::Drawable;
use crate::map::Floor;
use crate::monsters::Monster;
use crate::player::Player;

pub use blinding_light::*;
pub use magic_missle::*;
pub use slash::*;

use macroquad::prelude::*;

pub trait Attack: Drawable + Send + Sync {
	/// Just gives some information about the attack
	fn new(
		player: &Player, angle: f32, textures: &HashMap<String, Texture2D>, floor: &Floor,
		is_primary: bool,
	) -> Box<Self>
	where
		Self: Sized;
	/// If the attack has any side effects on the user, do them here
	fn side_effects(&self, player: &mut Player, floor: &Floor);
	fn mana_cost(&self) -> u16;
	// Returns whether or not the attack should be destroyed
	fn update(&mut self, monsters: &mut [Box<dyn Monster>], floor: &Floor) -> bool;
	fn cooldown(&self) -> u16;
}

pub fn update_attacks(
	monsters: &mut [Box<dyn Monster>], floor: &Floor, attacks: &mut Vec<Box<dyn Attack>>,
) {
	attacks.retain_mut(|attack| !attack.update(monsters, floor));
}
