use std::collections::HashMap;

use crate::attacks::*;
use crate::draw::Drawable;
use crate::map::{distance_squared, pos_to_tile, Floor, TILE_SIZE};
use crate::math::{AsAABB, AxisAlignedBoundingBox};
use crate::monsters::Monster;
use macroquad::prelude::*;

pub const PLAYER_SIZE: f32 = 12.0;

#[derive(Copy, Clone)]
pub enum PlayerClass {
	Warrior,
	Wizard,
}

pub struct Player {
	class: PlayerClass,
	pub angle: f32,
	pub pos: Vec2,
	speed: f32,
	health: f32,
	invincibility_frames: u16,
	pub primary_cooldown: u16,
	pub secondary_cooldown: u16,
	pub attacks: Vec<Box<dyn Attack>>,
}

impl Player {
	pub fn new(class: PlayerClass, pos: Vec2) -> Self {
		Self {
			class,
			pos,
			angle: 0.0,
			speed: 2.2,
			primary_cooldown: 0,
			secondary_cooldown: 0,
			health: 100.0,
			invincibility_frames: 0,
			attacks: Vec::with_capacity(2),
		}
	}

	#[inline]
	pub fn pos(&self) -> Vec2 {
		self.pos
	}

	#[inline]
	pub fn health(&self) -> f32 {
		self.health
	}

	#[inline]
	pub fn class(&self) -> PlayerClass {
		self.class
	}
}

impl AsAABB for Player {
	fn as_aabb(&self) -> AxisAlignedBoundingBox {
		AxisAlignedBoundingBox {
			pos: self.pos,
			size: Vec2::splat(PLAYER_SIZE),
		}
	}
}

impl Drawable for Player {
	fn pos(&self) -> Vec2 {
		self.pos
	}

	fn size(&self) -> Vec2 {
		Vec2::splat(PLAYER_SIZE)
	}

	fn draw(&self) {
		draw_rectangle(self.pos.x, self.pos.y, PLAYER_SIZE, PLAYER_SIZE, RED);
		draw_text(
			&self.health.to_string(),
			self.pos.x,
			self.pos.y - PLAYER_SIZE,
			12.0,
			WHITE,
		);
	}
}

pub fn move_player(player: &mut Player, angle: f32, speed: Option<Vec2>, floor: &Floor) {
	let direction: Vec2 = (angle.cos(), angle.sin()).into();
	let distance = direction * speed.unwrap_or_else(|| Vec2::splat(player.speed));

	if !floor.collision(player, distance) {
		player.pos += distance;
	}
}

pub fn damage_player(player: &mut Player, damage: f32, damage_direction: f32, floor: &Floor) {
	if player.invincibility_frames > 0 {
		return;
	}

	let new_health = player.health - damage;

	player.health = match new_health > 0.0 {
		true => new_health,
		false => 0.0,
	};

	// Have the player "flinch" away from damage
	move_player(
		player,
		damage_direction,
		Some(Vec2::splat(PLAYER_SIZE)),
		floor,
	);

	player.invincibility_frames = (damage as u16) * 2;
}

pub fn update_cooldowns(players: &mut [Player]) {
	players.iter_mut().for_each(|p| {
		p.primary_cooldown = p.primary_cooldown.saturating_sub(1);
		p.secondary_cooldown = p.secondary_cooldown.saturating_sub(1);

		p.invincibility_frames = p.invincibility_frames.saturating_sub(1);
	});
}

pub fn primary_attack(
	player: &mut Player, textures: &HashMap<String, Texture2D>, monsters: &mut [Box<dyn Monster>],
	floor: &Floor,
) -> Box<dyn Attack> {
	match player.class {
		PlayerClass::Warrior => Slash::new(player, player.angle, textures, floor, true),
		PlayerClass::Wizard => MagicMissile::new(player, player.angle, textures, floor, true),
	}
}

pub fn secondary_attack(
	player: &mut Player, textures: &HashMap<String, Texture2D>, monsters: &mut [Box<dyn Monster>],
	floor: &Floor,
) -> Box<dyn Attack> {
	match player.class {
		PlayerClass::Warrior => Stab::new(player, player.angle, textures, floor, false),
		PlayerClass::Wizard => BlindingLight::new(player, player.angle, textures, floor, false),
	}
}

pub enum DoorInteraction {
	Opening,
	Closing,
}

pub fn interact_with_door<A: AsAABB>(
	entity: &mut A, door_interaction: DoorInteraction, floor: &mut Floor,
	textures: &HashMap<String, Texture2D>,
) {
	// First, see if the player is in contact with a door
	let entity_tile_pos = pos_to_tile(entity);

	// Find all door that's within one tile distance of the player, then pick the closest one

	let door = floor
		.doors()
		.filter(|door| {
			let tile_distance = (door.tile_pos() - entity_tile_pos).abs();

			// You can't open or close doors that you're inside of
			tile_distance.cmple(IVec2::ONE).all() && !door.tile_pos().eq(&entity_tile_pos)
		})
		.reduce(|door_obj, door2_obj| {
			let door = &door_obj.door().unwrap();
			let door2 = &door_obj.door().unwrap();

			// First, depending on the action the player is taking, we can pretty easily decide of the player wants to open or close the door
			let door_will_be_affected = match door_interaction {
				DoorInteraction::Opening => !door.is_open,
				DoorInteraction::Closing => door.is_open,
			};

			let door2_will_be_affected = match door_interaction {
				DoorInteraction::Opening => !door2.is_open,
				DoorInteraction::Closing => door2.is_open,
			};

			if door_will_be_affected && door2_will_be_affected {
				let door_distance = (door.pos() * IVec2::splat(TILE_SIZE as i32))
					.as_vec2()
					.distance_squared(entity.center());
				let door2_distance = (door.pos() * IVec2::splat(TILE_SIZE as i32))
					.as_vec2()
					.distance_squared(entity.center());

				match door_distance < door2_distance {
					true => door_obj,
					false => door2_obj,
				}
			} else {
				if door_will_be_affected {
					door_obj
				} else {
					door2_obj
				}
			}
		});

	if let Some(door) = door {
		match door_interaction {
			DoorInteraction::Opening => door.open_door(textures),
			DoorInteraction::Closing => door.close_door(textures),
		};
	}
}
