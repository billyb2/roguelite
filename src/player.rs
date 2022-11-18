use std::collections::HashMap;
use std::fmt::Display;

use crate::attacks::*;
use crate::draw::Drawable;
use crate::enchantments::{Enchantable, Enchantment};
use crate::items::ItemType::*;
use crate::items::{attack_with_item, ItemInfo};
use crate::map::{pos_to_tile, Floor, TILE_SIZE};
use crate::math::{AsAABB, AxisAlignedBoundingBox};
use crate::monsters::Monster;
use macroquad::prelude::*;

pub const PLAYER_SIZE: f32 = 12.0;

#[derive(Copy, Clone)]
pub enum PlayerClass {
	Warrior,
	Wizard,
}

/// Info regarding points such as HP or MP
#[derive(Debug)]
struct PointInfo {
	/// Currently number of points
	points: u16,
	/// The number of frames until your points go up by 1, lower is better
	regen_rate: u16,
	max_points: u16,
	time_til_regen: u16,
}

impl Default for PointInfo {
	fn default() -> Self {
		Self {
			points: 0,
			regen_rate: 0,
			max_points: 0,
			time_til_regen: 0,
		}
	}
}

#[derive(Copy, Clone)]
pub enum Spell {
	BlindingLight,
	MagicMissile,
}

impl Display for Spell {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Spell::BlindingLight => "Blinding Light",
			Spell::MagicMissile => "Magic Missile",
		})
	}
}

pub struct Player {
	pub angle: f32,
	pub pos: Vec2,
	speed: f32,
	hp: PointInfo,
	mp: PointInfo,
	/// The ability to resist magical enchantments
	willpower: u16,
	invincibility_frames: u16,

	pub primary_item: Option<ItemInfo>,
	pub secondary_item: Option<ItemInfo>,

	pub primary_cooldown: u16,
	pub secondary_cooldown: u16,

	spells: Vec<Spell>,

	pub changing_spell: bool,
	pub time_til_change_spell: u8,
}

impl Player {
	pub fn new(class: PlayerClass, pos: Vec2) -> Self {
		let primary_item = Some(match class {
			PlayerClass::Warrior => ItemInfo::new(ShortSword),
			PlayerClass::Wizard => ItemInfo::new(WizardGlove),
		});

		let hp = match class {
			PlayerClass::Wizard => PointInfo {
				points: 20,
				max_points: 20,
				// 15 seconds
				regen_rate: 15 * 60,
				..Default::default()
			},
			PlayerClass::Warrior => PointInfo {
				points: 30,
				max_points: 30,
				// 15 seconds
				regen_rate: 15 * 60,
				..Default::default()
			},
		};

		let mp = match class {
			PlayerClass::Wizard => PointInfo {
				points: 5,
				max_points: 5,
				// 8 seconds
				regen_rate: 8 * 60,
				..Default::default()
			},
			PlayerClass::Warrior => PointInfo {
				points: 3,
				max_points: 3,
				// 10 seconds
				regen_rate: 10 * 60,
				..Default::default()
			},
		};

		let willpower = match class {
			PlayerClass::Wizard => 20,
			PlayerClass::Warrior => 10,
		};

		let spells = match class {
			PlayerClass::Warrior => Vec::new(),
			PlayerClass::Wizard => vec![Spell::MagicMissile, Spell::BlindingLight],
		};

		Self {
			pos,
			angle: 0.0,
			speed: 2.2,
			primary_cooldown: 0,
			secondary_cooldown: 0,
			hp,
			mp,
			willpower,
			invincibility_frames: 0,
			primary_item,
			secondary_item: None,
			spells,
			changing_spell: false,
			time_til_change_spell: 0,
		}
	}

	/// Replace the first and last items in the spells Vec
	pub fn cycle_spells(&mut self) {
		let spells_len = self.spells.len();
		self.spells.swap(0, spells_len - 1);
	}

	#[inline]
	pub fn pos(&self) -> Vec2 {
		self.pos
	}

	#[inline]
	pub fn hp(&self) -> u16 {
		self.hp.points
	}

	#[inline]
	pub fn mp(&self) -> u16 {
		self.mp.points
	}

	#[inline]
	pub fn spells(&self) -> &[Spell] {
		&self.spells
	}
}

pub fn move_player(player: &mut Player, angle: f32, speed: Option<Vec2>, floor: &Floor) {
	let direction: Vec2 = (angle.cos(), angle.sin()).into();
	let distance = direction * speed.unwrap_or_else(|| Vec2::splat(player.speed));

	if !floor.collision(player, distance) {
		player.pos += distance;
	}
}

pub fn damage_player(player: &mut Player, damage: u16, damage_direction: f32, floor: &Floor) {
	if player.invincibility_frames > 0 {
		return;
	}

	player.hp.points = player.hp.points.saturating_sub(damage);

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
	let regen = |point_info: &mut PointInfo| {
		if point_info.points < point_info.max_points {
			point_info.time_til_regen = point_info.time_til_regen.saturating_sub(1);

			if point_info.time_til_regen == 0 {
				point_info.points += 1;

				point_info.time_til_regen = point_info.regen_rate;
				return;
			}
		}
	};

	players.iter_mut().for_each(|p| {
		p.primary_cooldown = p.primary_cooldown.saturating_sub(1);
		p.secondary_cooldown = p.secondary_cooldown.saturating_sub(1);

		p.invincibility_frames = p.invincibility_frames.saturating_sub(1);

		p.time_til_change_spell = p.time_til_change_spell.saturating_sub(1);

		if p.changing_spell {
			if p.time_til_change_spell == 0 {
				if !p.spells.is_empty() {
					p.cycle_spells();
					p.changing_spell = false;
				}
			}
		}

		regen(&mut p.hp);
		regen(&mut p.mp);
	});
}

pub fn player_attack(
	player: &mut Player, textures: &HashMap<String, Texture2D>, attacks: &mut Vec<Box<dyn Attack>>,
	floor: &Floor, is_primary: bool,
) {
	let item = match is_primary {
		true => player.primary_item,
		false => player.secondary_item,
	};

	if let Some(item) = item {
		if let Some(attack) = attack_with_item(item, player, textures, floor, is_primary) {
			let cooldown = match is_primary {
				true => &mut player.primary_cooldown,
				false => &mut player.secondary_cooldown,
			};

			if *cooldown != 0 {
				return;
			}

			if player.mp.points >= attack.mana_cost() {
				player.mp.points -= attack.mana_cost();
			} else {
				return;
			}

			*cooldown = attack.cooldown();

			attacks.push(attack);
		}
	}
}

pub enum DoorInteraction {
	Opening,
	Closing,
}

pub fn interact_with_door<A: AsAABB>(
	entity: &A, players: &[Player], monsters: &[Box<dyn Monster>],
	door_interaction: DoorInteraction, floor: &mut Floor, textures: &HashMap<String, Texture2D>,
) {
	// First, see if the player is in contact with a door
	let entity_tile_pos = pos_to_tile(entity);

	// Find all door that's within one tile distance of the player, then pick the closest one

	let door = floor
		.doors()
		.filter(|door| {
			let tile_distance = (door.tile_pos() - entity_tile_pos).abs();

			let entity_in_door = monsters
				.iter()
				.map(|m| pos_to_tile(&m.as_aabb()))
				.chain(players.iter().map(|p| pos_to_tile(p)))
				.any(|pos| pos == door.tile_pos());

			// You can't open or close doors that you're inside of
			tile_distance.cmple(IVec2::ONE).all()
				&& !door.tile_pos().eq(&entity_tile_pos)
				&& !entity_in_door
		})
		.reduce(|door_obj, door2_obj| {
			let door = &door_obj.door().unwrap();
			let door2 = &door2_obj.door().unwrap();

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
				let door_distance = (door_obj.tile_pos() * IVec2::splat(TILE_SIZE as i32))
					.as_vec2()
					.distance_squared(entity.center());
				let door2_distance = (door2_obj.tile_pos() * IVec2::splat(TILE_SIZE as i32))
					.as_vec2()
					.distance_squared(entity.center());

				match door_distance < door2_distance {
					true => door_obj,
					false => door2_obj,
				}
			} else if door_will_be_affected {
				door_obj
			} else {
				door2_obj
			}
		});

	if let Some(door) = door {
		match door_interaction {
			DoorInteraction::Opening => door.open_door(textures),
			DoorInteraction::Closing => door.close_door(textures),
		};
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
			&self.hp.points.to_string(),
			self.pos.x,
			self.pos.y - PLAYER_SIZE,
			12.0,
			WHITE,
		);
	}
}

impl Enchantable for Player {
	fn apply_enchantment(&mut self, enchantment: Enchantment) {
		todo!()
	}

	fn update_enchantments(&mut self) {
		todo!()
	}
}
