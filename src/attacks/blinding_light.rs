use crate::draw::{Drawable, Textures};
use crate::enchantments::{Enchantment, EnchantmentKind};
use crate::map::{Floor, FloorInfo};
use crate::math::{aabb_collision, easy_polygon, AsPolygon, Polygon};
use crate::player::{Player, PLAYER_SIZE};
use macroquad::prelude::*;

use super::Attack;

const HALF_SIZE: Vec2 = Vec2::new(45.0, 45.0);
const SIZE: Vec2 = Vec2::new(90.0, 90.0);

pub struct BlindingLight {
	pos: Vec2,
	angle: f32,
	texture: Texture2D,
	time: u16,
}

impl Attack for BlindingLight {
	fn new(
		aabb: &dyn AsPolygon, _index: Option<usize>, angle: f32, textures: &Textures,
		_floor: &Floor, _is_primary: bool,
	) -> Box<Self> {
		Box::new(Self {
			pos: aabb.center() + (Vec2::new(angle.cos(), angle.sin()) * PLAYER_SIZE),
			angle,
			texture: *textures.get("blinding_light.webp").unwrap(),
			time: 0,
		})
	}

	fn update(&mut self, floor: &mut FloorInfo, _players: &mut [Player]) -> bool {
		self.time += 1;

		if self.time >= 60 {
			return true;
		}

		// Check to see if it's collided with a monster
		floor
			.monsters
			.iter_mut()
			.filter(|m| aabb_collision(self, &m.as_polygon(), Vec2::ZERO))
			.for_each(|monster| {
				monster.apply_enchantment(Enchantment {
					kind: EnchantmentKind::Blinded,
					strength: 0,
				});
			});

		false
	}

	fn cooldown(&self) -> u16 { 60 }

	fn mana_cost(&self) -> u16 { 3 }

	fn side_effects(&self, _player: &mut Player, _floor: &Floor) {}

	fn as_polygon_optional(&self) -> Option<Polygon> { Some(self.as_polygon()) }
}

impl AsPolygon for BlindingLight {
	fn as_polygon(&self) -> Polygon { easy_polygon(self.pos + HALF_SIZE, HALF_SIZE, self.angle) }
}

impl Drawable for BlindingLight {
	fn pos(&self) -> Vec2 { self.pos }

	fn size(&self) -> Vec2 { SIZE }

	fn rotation(&self) -> f32 { self.angle }

	fn texture(&self) -> Option<Texture2D> { Some(self.texture) }
}
