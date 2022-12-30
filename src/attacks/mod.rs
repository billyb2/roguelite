mod blinding_light;
mod magic_missle;
mod slash;
mod slimeball;
mod stab;
mod throwing_knife;

use crate::draw::{Drawable, Textures};
use crate::map::{Floor, FloorInfo};

use crate::math::AsAABB;
use crate::player::Player;

pub use blinding_light::*;
pub use magic_missle::*;
pub use slash::*;
pub use slimeball::*;
pub use stab::*;
pub use throwing_knife::*;

use macroquad::prelude::*;

pub trait Attack: Drawable + Send + Sync {
	/// Just gives some information about the attack
	fn new(
		player: &dyn AsAABB, player_index: Option<usize>, angle: f32, textures: &Textures,
		floor: &Floor, is_primary: bool,
	) -> Box<Self>
	where
		Self: Sized;
	/// If the attack has any side effects on the user, do them here
	fn side_effects(&self, player: &mut Player, floor: &Floor);
	fn mana_cost(&self) -> u16;
	// Returns whether or not the attack should be destroyed
	fn update(&mut self, floor: &mut FloorInfo, players: &mut [Player]) -> bool;
	fn cooldown(&self) -> u16;
}

pub fn update_attacks(
	players: &mut [Player], floor: &mut FloorInfo, attacks: &mut Vec<Box<dyn Attack>>,
) {
	attacks.retain_mut(|attack| !attack.update(floor, players));
}
