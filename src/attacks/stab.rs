use crate::draw::{load_my_image, Drawable};
use crate::map::{Floor, FloorInfo};
use crate::math::{aabb_collision, easy_polygon, get_angle, AsPolygon, Polygon};
use crate::player::{DamageInfo, Player, PLAYER_SIZE};
use macroquad::prelude::*;

use super::Attack;

const HALF_SIZE: Vec2 = Vec2::new(7.5, 2.5);
const SIZE: Vec2 = Vec2::new(15.0, 5.0);

pub struct Stab {
	pos: Vec2,
	angle: f32,
	texture: Texture2D,
	time: u16,
	player_index: usize,
	num_piercings: u8,
}

impl Attack for Stab {
	fn new(
		aabb: &dyn AsPolygon, index: Option<usize>, angle: f32, _floor: &Floor, _is_primary: bool,
	) -> Box<Self> {
		Box::new(Self {
			pos: aabb.center(),
			angle,
			texture: load_my_image("stab.webp"),
			time: 0,
			player_index: index.unwrap(),
			num_piercings: 0,
		})
	}

	fn side_effects(&self, player: &mut Player, floor: &Floor) {
		let change = Vec2::new(self.angle.cos(), self.angle.sin()) * PLAYER_SIZE;

		if !floor.collision(player, change) {
			player.pos += change;
		}
	}

	fn update(&mut self, floor_info: &mut FloorInfo, _players: &mut [Player]) -> bool {
		let movement = Vec2::new(self.angle.cos(), self.angle.sin()) * 6.0;

		self.pos += movement;
		self.time += 1;

		if self.time >= 6 {
			return true;
		}

		let aabb = self.as_polygon();

		// Check to see if it's collided with a monster
		if let Some(monster) = floor_info
			.monsters
			.iter_mut()
			.find(|m| aabb_collision(&aabb, &m.as_polygon(), Vec2::ZERO))
		{
			// Damage is low bc of hitting enemies multiple times
			const DAMAGE: u16 = 25;

			let direction = get_angle(monster.pos(), self.pos);
			let damage_info = DamageInfo {
				damage: DAMAGE,
				direction,
				player: self.player_index,
			};

			monster.take_damage(damage_info, &floor_info.floor);

			return true;
		}

		false
	}

	fn cooldown(&self) -> u16 { 50 }

	fn mana_cost(&self) -> u16 { 0 }
}

impl AsPolygon for Stab {
	fn as_polygon(&self) -> Polygon { easy_polygon(self.pos + HALF_SIZE, HALF_SIZE, self.angle) }
}

impl Drawable for Stab {
	fn pos(&self) -> Vec2 { self.pos }

	fn size(&self) -> Vec2 { SIZE }

	fn rotation(&self) -> f32 { self.angle }

	fn texture(&self) -> Option<Texture2D> { Some(self.texture) }
}
