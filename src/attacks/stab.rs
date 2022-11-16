use std::collections::HashMap;

use crate::draw::Drawable;
use crate::map::Floor;
use crate::math::{aabb_collision, get_angle, AsAABB, AxisAlignedBoundingBox};
use crate::monsters::Monster;
use crate::player::Player;
use macroquad::prelude::*;

use super::Attack;

const SIZE: Vec2 = Vec2::new(25.0, 5.0);

pub struct Stab {
	pos: Vec2,
	angle: f32,
	texture: Texture2D,
	time: u16,
}

impl Attack for Stab {
	fn new(
		player: &mut Player, angle: f32, textures: &HashMap<String, Texture2D>, _floor: &Floor,
		is_primary: bool,
	) -> Box<Self> {
		Box::new(Self {
			pos: player.pos(),
			angle,
			texture: *textures.get("stab.webp").unwrap(),
			time: 0,
		})
	}

	fn update(&mut self, monsters: &mut [Box<dyn Monster>], floor: &Floor) -> bool {
		let movement = Vec2::new(self.angle.cos(), self.angle.sin()) * 9.0;

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
		monsters
			.iter_mut()
			.filter(|m| aabb_collision(self, &m.as_aabb(), Vec2::ZERO))
			.for_each(|monster| {
				const DAMAGE: f32 = 11.0;

				let damage_direction = get_angle(monster.pos(), self.pos);
				monster.take_damage(DAMAGE, damage_direction, floor);
			});

		false
	}

	fn cooldown(&self) -> u16 {
		45
	}
}

impl AsAABB for Stab {
	fn as_aabb(&self) -> AxisAlignedBoundingBox {
		AxisAlignedBoundingBox {
			pos: self.pos,
			size: SIZE,
		}
	}
}

impl Drawable for Stab {
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
