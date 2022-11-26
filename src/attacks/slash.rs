use crate::draw::{Drawable, Textures};
use crate::map::{Floor, FloorInfo};
use crate::math::{aabb_collision, get_angle, AsAABB, AxisAlignedBoundingBox};
use crate::player::{DamageInfo, Player, PLAYER_SIZE};
use macroquad::prelude::*;

use super::Attack;

const SIZE: Vec2 = Vec2::new(10.0, 15.0);

pub struct Slash {
	pos: Vec2,
	angle: f32,
	texture: Texture2D,
	time: u16,
	player_index: usize,
	num_piercings: u8,
}

impl Attack for Slash {
	fn new(
		aabb: &dyn AsAABB, index: Option<usize>, angle: f32, textures: &Textures, _floor: &Floor,
		_is_primary: bool,
	) -> Box<Self> {
		Box::new(Self {
			pos: aabb.center(),
			angle,
			texture: *textures.get("slash.webp").unwrap(),
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

		if !floor_info.floor.collision(self, movement) {
			self.pos += movement;
			self.time += 1;
		} else {
			return true;
		}

		if self.time >= 10 {
			return true;
		}

		if self.num_piercings >= 6 {
			return true;
		}

		let aabb = self.as_aabb();

		// Check to see if it's collided with a monster
		floor_info
			.monsters
			.iter_mut()
			.filter(|m| aabb_collision(&aabb, &m.as_aabb(), Vec2::ZERO))
			.for_each(|monster| {
				// Damage is low bc of hitting enemies multiple times
				const DAMAGE: u16 = 5;

				let direction = get_angle(monster.pos(), self.pos);
				let damage_info = DamageInfo {
					damage: DAMAGE,
					direction,
					player: self.player_index,
				};

				monster.take_damage(damage_info, &floor_info.floor);

				self.num_piercings += 1;
			});

		false
	}

	fn cooldown(&self) -> u16 {
		30
	}

	fn mana_cost(&self) -> u16 {
		0
	}
}

impl AsAABB for Slash {
	fn as_aabb(&self) -> AxisAlignedBoundingBox {
		AxisAlignedBoundingBox {
			pos: self.pos,
			size: SIZE,
		}
	}
}

impl Drawable for Slash {
	fn pos(&self) -> Vec2 {
		self.pos
	}

	fn size(&self) -> Vec2 {
		SIZE
	}

	fn rotation(&self) -> f32 {
		self.angle
	}

	fn texture(&self) -> Option<Texture2D> {
		Some(self.texture)
	}
}
