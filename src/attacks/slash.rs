use std::f32::consts::PI;

use crate::draw::{Drawable, Textures};
use crate::map::{Floor, FloorInfo};
use crate::math::{aabb_collision, easy_polygon, get_angle, AsPolygon};
use crate::player::{DamageInfo, Player, PLAYER_SIZE};
use macroquad::prelude::*;

use super::Attack;

const HALF_SIZE: Vec2 = Vec2::new(15.0 * 0.5, 20.0 * 0.5);
const SIZE: Vec2 = Vec2::new(15.0, 20.0);
const SWING_TIME: u16 = 10;

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
		aabb: &dyn AsPolygon, index: Option<usize>, angle: f32, textures: &Textures,
		_floor: &Floor, _is_primary: bool,
	) -> Box<Self> {
		let angle = angle + (PI * 0.33);
		Box::new(Self {
			pos: aabb.center(),
			angle,
			texture: *textures.get("sword.webp").unwrap(),
			time: 0,
			player_index: index.unwrap(),
			num_piercings: 0,
		})
	}

	fn side_effects(&self, _player: &mut Player, _floor: &Floor) {}

	fn update(&mut self, floor_info: &mut FloorInfo, players: &mut [Player]) -> bool {
		self.time += 1;

		if self.time >= SWING_TIME {
			return true;
		}

		self.angle -= 0.2;
		let movement = Vec2::new(self.angle.cos(), self.angle.sin()) * PLAYER_SIZE * 2.0;

		self.pos = players[self.player_index].center() + movement;

		let poly = self.as_polygon();

		// Check to see if it's collided with a monster
		floor_info
			.monsters
			.iter_mut()
			.filter(|m| aabb_collision(&poly, &m.as_polygon(), Vec2::ZERO))
			.for_each(|monster| {
				// Damage is low bc of hitting enemies multiple times
				const DAMAGE: u16 = 4;

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

	fn cooldown(&self) -> u16 { SWING_TIME * 3 }

	fn mana_cost(&self) -> u16 { 0 }
}

impl AsPolygon for Slash {
	fn as_polygon(&self) -> crate::math::Polygon {
		easy_polygon(self.pos + HALF_SIZE, HALF_SIZE, 0.0)
	}
}

impl Drawable for Slash {
	fn pos(&self) -> Vec2 { self.pos }

	fn size(&self) -> Vec2 { SIZE }

	fn rotation(&self) -> f32 { self.angle }

	fn flip_x(&self) -> bool { false }

	fn texture(&self) -> Option<Texture2D> { Some(self.texture) }
}
