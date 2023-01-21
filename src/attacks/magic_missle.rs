use crate::draw::{load_my_image, Drawable};
use crate::map::{Floor, FloorInfo};
use crate::math::{aabb_collision_dir, easy_polygon, get_angle, AsPolygon, Polygon};
use crate::player::{DamageInfo, Player};
use macroquad::prelude::*;
use serde::Serialize;

use super::Attack;

const HALF_SIZE: Vec2 = Vec2::new(7.5, 7.5);
const SIZE: Vec2 = Vec2::new(15.0, 15.0);

#[derive(Clone, Serialize)]
pub struct MagicMissile {
	pos: Vec2,
	angle: f32,
	time: u16,
	bounces: u16,
	player_index: usize,
}

impl Attack for MagicMissile {
	fn new(
		aabb: &dyn AsPolygon, index: Option<usize>, angle: f32, _floor: &Floor, _is_primary: bool,
	) -> Self {
		Self {
			pos: aabb.center(),
			angle,
			time: 0,
			bounces: 0,
			player_index: index.unwrap(),
		}
	}

	fn side_effects(&self, player: &mut Player, floor: &Floor) {
		// "Knocback" the player a bit
		let change = -Vec2::new(self.angle.cos(), self.angle.sin()) * 1.5;

		if !floor.collision(player, change) {
			player.pos += change;
		}
	}

	fn update(&mut self, floor_info: &mut FloorInfo, _players: &mut [Player]) -> bool {
		let mut movement = Vec2::new(self.angle.cos(), self.angle.sin()) * 5.0;

		let collision_info = floor_info.floor.collision_dir(self, movement);

		if collision_info.x {
			movement.x = -movement.x;
		}

		if collision_info.y {
			movement.y = -movement.y;
		}

		if collision_info.any() {
			if self.bounces < 3 {
				self.bounces += 1;
			}
		}

		self.angle = get_angle(movement, Vec2::ZERO);

		// Check to see if it's collided with a monster
		if let Some((monster, collision_info)) = floor_info.monsters.iter_mut().find_map(|m| {
			let collision_info = aabb_collision_dir(self, &m.as_polygon(), Vec2::ZERO);

			if collision_info.any() {
				Some((m, collision_info))
			} else {
				None
			}
		}) {
			const BASE_DAMAGE: u16 = 1;
			// The damage increases the more the projectile bounces
			let damage = BASE_DAMAGE.pow((1 + self.bounces).into());

			let direction = get_angle(monster.pos(), self.pos);

			let damage_info = DamageInfo {
				damage,
				direction,
				player: self.player_index,
			};
			monster.take_damage(damage_info, &floor_info.floor);

			if self.bounces > 0 {
				if collision_info.x {
					movement.x = -movement.x;
				}

				if collision_info.y {
					movement.y = -movement.y;
				}

				self.angle = get_angle(movement, Vec2::ZERO);

				if self.bounces < 3 {
					self.bounces += 1;
				}
			} else {
				self.pos -= movement;
				self.time += 1;
			}
		}

		self.pos += movement;
		self.time += 1;

		if self.time >= 60 {
			return true;
		}

		false
	}

	fn cooldown(&self) -> u16 { 45 }

	fn mana_cost(&self) -> u16 { 1 }

	fn as_polygon_optional(&self) -> Option<Polygon> { Some(self.as_polygon()) }
}

impl AsPolygon for MagicMissile {
	fn as_polygon(&self) -> Polygon { easy_polygon(self.pos + HALF_SIZE, HALF_SIZE, self.angle) }
}

impl Drawable for MagicMissile {
	fn pos(&self) -> Vec2 { self.pos }

	fn size(&self) -> Vec2 { SIZE }

	fn rotation(&self) -> f32 { self.angle }

	fn texture(&self) -> Option<Texture2D> { Some(load_my_image("magic_missile.webp")) }
}
