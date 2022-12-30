use crate::draw::{Drawable, Textures};
use crate::items::{ItemInfo, ItemType};
use crate::map::{pos_to_tile, Floor, FloorInfo};
use crate::math::{aabb_collision, get_angle, AsAABB, AxisAlignedBoundingBox};
use crate::player::{DamageInfo, Player, PLAYER_SIZE};
use macroquad::prelude::*;

use super::Attack;

const SIZE: Vec2 = Vec2::new(15.0, 30.0);

pub struct ThrownKnife {
	pos: Vec2,
	angle: f32,
	drawn_angle: f32,
	texture: Texture2D,
	time: u16,
	player_index: usize,
}

impl Attack for ThrownKnife {
	fn new(
		aabb: &dyn AsAABB, index: Option<usize>, angle: f32, textures: &Textures, _floor: &Floor,
		_is_primary: bool,
	) -> Box<Self> {
		Box::new(Self {
			pos: aabb.center() + Vec2::new(angle.cos(), angle.sin()) * SIZE * 0.5,
			angle,
			drawn_angle: angle,
			texture: *textures.get("throwing_knife.webp").unwrap(),
			time: 0,
			player_index: index.unwrap(),
		})
	}

	fn side_effects(&self, player: &mut Player, floor: &Floor) {
		let change = Vec2::new(self.angle.cos(), self.angle.sin()) * PLAYER_SIZE;

		if !floor.collision(player, change) {
			player.pos += change;
		}
	}

	fn update(&mut self, floor_info: &mut FloorInfo, _players: &mut [Player]) -> bool {
		let movement = Vec2::new(self.angle.cos(), self.angle.sin()) * 8.0;
		let mut should_drop = false;

		if !floor_info.floor.collision(self, movement) {
			self.pos += movement;
			self.time += 1;
		} else {
			should_drop = true;
		}

		self.drawn_angle += 0.5;

		let aabb = self.as_aabb();

		// Check to see if it's collided with a monster
		if let Some(monster) = floor_info
			.monsters
			.iter_mut()
			.find(|m| aabb_collision(&aabb, &m.as_aabb(), Vec2::ZERO))
		{
			const DAMAGE: u16 = 18;

			let direction = get_angle(monster.pos(), self.pos);
			let damage_info = DamageInfo {
				damage: DAMAGE,
				direction,
				player: self.player_index,
			};

			monster.take_damage(damage_info, &floor_info.floor);

			should_drop = true;
		}

		if should_drop {
			let should_break = rand::gen_range(0, 9) == 9;

			// Don't drop anything if the item should break
			if !should_break {
				// Find the nearest tile that is open, or just break
				let tile_pos = pos_to_tile(self);
				let tile_pos_vec2 = tile_pos.as_vec2();

				if let Some(item_pos) = [
					IVec2::ZERO,
					IVec2::new(-1, 0),
					IVec2::new(0, -1),
					IVec2::new(-1, -1),
					IVec2::new(1, 0),
					IVec2::new(0, 1),
					IVec2::new(1, 1),
					IVec2::new(-1, 1),
					IVec2::new(1, -1),
				]
				.into_iter()
				.map(|change| tile_pos + change)
				.filter(
					|tile_pos| match floor_info.floor.get_object_from_pos(*tile_pos) {
						Some(object) => !object.is_collidable(),
						None => false,
					},
				)
				.reduce(|tile_pos1, tile_pos2| {
					let distance1 = tile_pos1.as_vec2().distance_squared(tile_pos_vec2);
					let distance2 = tile_pos2.as_vec2().distance_squared(tile_pos_vec2);

					match distance1 < distance2 {
						true => tile_pos1,
						false => tile_pos2,
					}
				}) {
					let item = ItemInfo::new(ItemType::ThrowingKnife, Some(item_pos));
					floor_info.floor.add_item_to_object(item);
				}
			}
		}

		should_drop
	}

	fn cooldown(&self) -> u16 {
		10
	}

	fn mana_cost(&self) -> u16 {
		0
	}
}

impl AsAABB for ThrownKnife {
	fn as_aabb(&self) -> AxisAlignedBoundingBox {
		AxisAlignedBoundingBox {
			pos: self.pos,
			size: Vec2::splat(SIZE.max_element()),
		}
	}
}

impl Drawable for ThrownKnife {
	fn pos(&self) -> Vec2 {
		self.pos
	}

	fn size(&self) -> Vec2 {
		SIZE
	}

	fn rotation(&self) -> f32 {
		self.drawn_angle
	}

	fn texture(&self) -> Option<Texture2D> {
		Some(self.texture)
	}
}
