use std::collections::HashMap;

use crate::draw::Drawable;
use crate::map::Floor;
use crate::math::{aabb_collision, get_angle, AsAABB, AxisAlignedBoundingBox};
use crate::monsters::Monster;
use crate::player::{Player, PLAYER_SIZE};
use macroquad::prelude::*;

use super::Attack;

const SIZE: Vec2 = Vec2::new(15.0, 25.0);

pub struct Slash {
	pos: Vec2,
	angle: f32,
	texture: Texture2D,
	time: u16,
}

impl Attack for Slash {
	fn new(
		player: &Player, angle: f32, textures: &HashMap<String, Texture2D>, floor: &Floor,
		_is_primary: bool,
	) -> Box<Self> {
		Box::new(Self {
			pos: player.pos(),
			angle,
			texture: *textures.get("slash.webp").unwrap(),
			time: 0,
		})
	}

	fn side_effects(&self, player: &mut Player, floor: &Floor) {
		let change = Vec2::new(self.angle.cos(), self.angle.sin()) * PLAYER_SIZE;

		if !floor.collision(player, change) {
			player.pos += change;
		}
	}

	fn update(&mut self, monsters: &mut [Box<dyn Monster>], floor: &Floor) -> bool {
		let movement = Vec2::new(self.angle.cos(), self.angle.sin()) * 5.0;

		if !floor.collision(self, movement) {
			self.pos += movement;
			self.time += 1;
		} else {
			return true;
		}

		if self.time >= 10 {
			return true;
		}

		// Check to see if it's collided with a monster
		if let Some(monster) = monsters
			.iter_mut()
			.find(|m| aabb_collision(self, &m.as_aabb(), Vec2::ZERO))
		{
			const DAMAGE: u16 = 10;

			let damage_direction = get_angle(monster.pos(), self.pos);
			monster.take_damage(DAMAGE, damage_direction, floor);

			return true;
		}

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
