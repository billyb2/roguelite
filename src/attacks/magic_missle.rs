use std::collections::HashMap;

use crate::draw::Drawable;
use crate::map::Floor;
use crate::math::{aabb_collision, get_angle, AsAABB, AxisAlignedBoundingBox};
use crate::monsters::Monster;
use crate::player::{DamageInfo, Player};
use macroquad::prelude::*;

use super::Attack;

const SIZE: Vec2 = Vec2::new(15.0, 15.0);

pub struct MagicMissile {
	pos: Vec2,
	angle: f32,
	texture: Texture2D,
	time: u16,
	bounces: u16,
	player_index: usize,
}

impl Attack for MagicMissile {
	fn new(
		player: &Player, angle: f32, textures: &HashMap<String, Texture2D>, floor: &Floor,
		_is_primary: bool,
	) -> Box<Self> {
		Box::new(Self {
			pos: player.pos(),
			angle,
			texture: *textures.get("magic_missile.webp").unwrap(),
			time: 0,
			bounces: 0,
			player_index: player.index(),
		})
	}

	fn side_effects(&self, player: &mut Player, floor: &Floor) {
		// "Knocback" the player a bit
		let change = -Vec2::new(self.angle.cos(), self.angle.sin()) * 1.5;

		if !floor.collision(player, change) {
			player.pos += change;
		}
	}

	fn update(&mut self, monsters: &mut [Box<dyn Monster>], floor: &Floor) -> bool {
		let movement = Vec2::new(self.angle.cos(), self.angle.sin()) * 5.0;

		if let Some(object) = floor.collision_obj(self, movement) {
			let object_center = object.pos() + (object.size() / 2.0);

			self.angle = get_angle(self.pos, object_center);
			self.pos += Vec2::new(self.angle.cos(), self.angle.sin()) * 5.0;

			self.bounces += 1;

			if self.bounces > 3 {
				self.bounces = 3;
			}
		} else {
			self.pos += movement;
			self.time += 1;
		}

		if self.time >= 60 {
			return true;
		}

		// Check to see if it's collided with a monster
		if let Some(monster) = monsters
			.iter_mut()
			.find(|m| aabb_collision(self, &m.as_aabb(), Vec2::ZERO))
		{
			const BASE_DAMAGE: u16 = 4;
			// The damage increases the more the projectile bounces
			let damage = BASE_DAMAGE.pow((1 + self.bounces).into());

			let direction = get_angle(monster.pos(), self.pos);

			let damage_info = DamageInfo {
				damage,
				direction,
				player: self.player_index,
			};
			monster.take_damage(damage_info, floor);

			self.angle = get_angle(self.pos, monster.pos());
			self.pos += Vec2::new(self.angle.cos(), self.angle.sin()) * 5.0;
		}

		false
	}

	fn cooldown(&self) -> u16 {
		45
	}

	fn mana_cost(&self) -> u16 {
		1
	}
}

impl AsAABB for MagicMissile {
	fn as_aabb(&self) -> AxisAlignedBoundingBox {
		AxisAlignedBoundingBox {
			pos: self.pos,
			size: SIZE,
		}
	}
}

impl Drawable for MagicMissile {
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
