use crate::draw::{load_my_image, Drawable, Textures};
use crate::enchantments::{Enchantable, Enchantment, EnchantmentKind};
use crate::map::{Floor, FloorInfo};
use crate::math::{aabb_collision, easy_polygon, get_angle, AsPolygon, Polygon};
use crate::player::{damage_player, Player};
use macroquad::prelude::*;

use super::Attack;

const HALF_SIZE: Vec2 = Vec2::new(7.5, 2.5);
const SIZE: Vec2 = Vec2::new(15.0, 5.0);

pub struct Slimeball {
	pos: Vec2,
	angle: f32,
	texture: Texture2D,
	time: u16,
}

impl Attack for Slimeball {
	fn new(
		aabb: &dyn AsPolygon, _index: Option<usize>, angle: f32, _floor: &Floor, _is_primary: bool,
	) -> Box<Self> {
		Box::new(Self {
			pos: aabb.center(),
			angle,
			texture: load_my_image("slimeball.webp"),
			time: 0,
		})
	}

	fn side_effects(&self, _player: &mut Player, _floor_info: &Floor) {}

	fn update(&mut self, floor_info: &mut FloorInfo, players: &mut [Player]) -> bool {
		let movement = Vec2::new(self.angle.cos(), self.angle.sin()) * 2.2;

		if !floor_info.floor.collision(self, movement) {
			self.pos += movement;
			self.time += 1;
		} else {
			return true;
		}

		if self.time >= 30 {
			return true;
		}

		let poly = self.as_polygon();

		// Check to see if it's collided with a player
		if let Some(player) = players
			.iter_mut()
			.find(|p| aabb_collision(&poly, &p.as_polygon(), Vec2::ZERO))
		{
			const DAMAGE: u16 = 6;

			let direction = get_angle(player.pos(), self.pos);

			damage_player(player, DAMAGE, direction, &floor_info.floor);
			player.apply_enchantment(Enchantment {
				kind: EnchantmentKind::Sticky,
				strength: 2,
			});

			return true;
		}

		false
	}

	fn cooldown(&self) -> u16 { 80 }

	fn mana_cost(&self) -> u16 { 0 }
}

impl AsPolygon for Slimeball {
	fn as_polygon(&self) -> Polygon { easy_polygon(self.pos + HALF_SIZE, HALF_SIZE, 0.0) }
}

impl Drawable for Slimeball {
	fn pos(&self) -> Vec2 { self.pos }

	fn size(&self) -> Vec2 { SIZE }

	fn rotation(&self) -> f32 { self.angle }

	fn texture(&self) -> Option<Texture2D> { Some(self.texture) }
}
