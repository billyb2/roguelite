use std::collections::{HashMap, HashSet};

use crate::attacks::{Attack, AttackObj, Slimeball};
use crate::draw::{load_my_image, Drawable};
use crate::enchantments::{Enchantable, Enchantment, EnchantmentKind};
use crate::map::{pos_to_tile, Floor, Object, TILE_SIZE};
use crate::math::{aabb_collision, easy_polygon, get_angle, AsPolygon, Polygon};
use crate::monsters::Monster;
use crate::player::{damage_player, DamageInfo, Player};

use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;
use serde::Serialize;

use super::Effect;

#[derive(PartialEq, Clone, Serialize)]
enum AttackMode {
	Passive,
	Attacking,
}

#[derive(Copy, Clone, Serialize)]
enum Target {
	Pos(Vec2),
}

const SIZE: f32 = 14.0;
const MAX_HEALTH: u16 = 15;

#[derive(Clone, Serialize)]
pub struct GreenSlime {
	health: u16,
	pos: Vec2,
	attack_mode: AttackMode,
	current_path: Option<(Vec<Vec2>, usize)>,
	enchantments: HashMap<EnchantmentKind, Effect>,
	// All the players who have damaged me
	damaged_by: HashSet<usize>,
	// Gotta keep track of if the target moved, to reset the path
	current_target: Option<Target>,
	time_til_attack: u8,
}

impl Monster for GreenSlime {
	fn new(pos: Vec2) -> Self {
		Self {
			pos,
			health: MAX_HEALTH,
			attack_mode: AttackMode::Passive,
			current_path: None,
			current_target: None,
			enchantments: HashMap::new(),
			damaged_by: HashSet::new(),
			time_til_attack: 30,
		}
	}

	fn movement(&mut self, players: &[Player], floor: &Floor) {
		match self.attack_mode {
			AttackMode::Passive => passive_mode(self, players, floor),
			AttackMode::Attacking => attack_mode(self, players, floor),
		};
	}

	fn attack(&mut self, players: &[Player], floor: &Floor, attacks: &mut Vec<AttackObj>) {
		self.time_til_attack = self.time_til_attack.saturating_sub(1);

		if self.time_til_attack > 0 {
			return;
		}

		let visible_objects = floor.visible_objects(self, Some(10));

		// Throw a slimeball at all visible players
		let players_to_attack = players.iter().filter(|player| {
			let player_tile_pos = pos_to_tile(&player.as_polygon());
			visible_objects
				.iter()
				.any(|obj| obj.tile_pos() == player_tile_pos)
		});

		players_to_attack.for_each(|player| {
			let angle = get_angle(player.center(), self.center());
			let slimeball = Slimeball::new(self, None, angle, &floor, true);

			self.time_til_attack = slimeball.cooldown() as u8;
			attacks.push(AttackObj::Slimeball(slimeball));
		});
	}

	fn damage_players(&mut self, players: &mut [Player], floor: &Floor) {
		players.iter_mut().for_each(|p| {
			if aabb_collision(p, self, Vec2::ZERO) {
				const DAMAGE: u16 = 10;
				let damage_direction = get_angle(p.pos(), self.pos);

				damage_player(p, DAMAGE, damage_direction, floor);
			}
		});
	}

	fn take_damage(&mut self, damage_info: DamageInfo, _floor: &Floor) {
		self.health = self.health.saturating_sub(damage_info.damage);
		self.damaged_by.insert(damage_info.player);
	}

	fn living(&self) -> bool { self.health > 0 }

	fn xp(&self) -> (&HashSet<usize>, u32) {
		const DEFAULT_XP: u32 = 2;
		(&self.damaged_by, DEFAULT_XP)
	}
}

fn step_pathfinding(my_monster: &mut GreenSlime, _players: &[Player], floor: &Floor, speed: f32) {
	if let Some((path, i)) = &mut my_monster.current_path {
		if let Some(pos) = path.get(*i) {
			let distance_to_target = my_monster.pos.distance(*pos);

			if speed >= distance_to_target {
				my_monster.pos = *pos;
				*i += 1;
			} else {
				let angle = get_angle(*pos, my_monster.pos);
				let change = Vec2::new(angle.cos(), angle.sin()) * speed;

				my_monster.pos += change;
			}
		} else {
			// Finished following path
			my_monster.current_path = None;
			my_monster.current_target = None;
		}
	} else {
		if let Some(Target::Pos(pos)) = my_monster.current_target {
			let poly = easy_polygon(
				pos + Vec2::splat((TILE_SIZE / 2) as f32),
				Vec2::splat((TILE_SIZE / 2) as f32),
				0.0,
			);

			let path = floor.find_path(my_monster, &poly, false, true, None);

			if let Some(path) = path {
				my_monster.current_path = Some((path, 1));
			} else {
				my_monster.current_path = None;
				my_monster.current_target = None;
			}
		}
	}
}

fn attack_mode(my_monster: &mut GreenSlime, players: &[Player], floor: &Floor) {
	// Check how far the closest player is
	let (player, p_distance) = players
		.iter()
		.map(|player| (player, player.center().distance(my_monster.center())))
		.reduce(|(p1, p1_distance), (p2, p2_distance)| {
			if p1_distance < p2_distance {
				(p1, p1_distance)
			} else {
				(p2, p2_distance)
			}
		})
		.unwrap();

	if p_distance <= (TILE_SIZE * 4) as f32 {
		// If the player is within 4 tiles, flee
		let valid_objs = floor
			.objects()
			.iter()
			.filter(|obj| match obj.is_collidable() {
				true => obj.door().is_some(),
				false => true,
			})
			.filter(|obj| obj.center().distance(player.center()) >= (TILE_SIZE * 4) as f32)
			.collect::<Vec<&Object>>();

		let obj = valid_objs.choose().unwrap();
		my_monster.current_target = Some(Target::Pos(obj.pos()));
	}

	step_pathfinding(my_monster, players, floor, 1.3);
}

fn passive_mode(my_monster: &mut GreenSlime, players: &[Player], floor: &Floor) {
	// Check if any players are in my visible range
	let visible_objects = floor.visible_objects(my_monster, Some(10));

	let should_aggro = players.iter().any(|player| {
		let player_tile_pos = pos_to_tile(player);

		visible_objects
			.iter()
			.any(|obj| obj.tile_pos() == player_tile_pos)
	});

	if should_aggro {
		my_monster.attack_mode = AttackMode::Attacking;
		return;
	}

	if my_monster.current_target.is_none() {
		// Choose a random room
		let valid_rooms = floor
			.objects()
			.iter()
			.filter(|obj| !obj.is_collidable())
			.collect::<Vec<&Object>>();

		let room = valid_rooms.choose().unwrap();

		let room_center_pos = room.center();
		my_monster.current_target = Some(Target::Pos(room_center_pos));
	}

	step_pathfinding(my_monster, players, floor, 1.0);
}

impl Enchantable for GreenSlime {
	fn apply_enchantment(&mut self, enchantment: Enchantment) {
		match enchantment.kind {
			// Lacking eyes, slimes can't be blinded, and will instead take 1 damage
			EnchantmentKind::Blinded => {
				self.health -= 1;
			},
			// I am a slime, lol
			EnchantmentKind::Sticky => (),
			EnchantmentKind::Regenerating => {
				self.enchantments.insert(
					enchantment.kind,
					Effect {
						enchantment,
						frames_left: 300,
					},
				);
			},
		};
	}

	fn update_enchantments(&mut self) {
		self.enchantments.retain(|e_kind, effect| {
			match e_kind {
				EnchantmentKind::Blinded => {
					self.attack_mode = AttackMode::Passive;
					self.current_target = None;
					self.current_path = None;
				},
				EnchantmentKind::Sticky => (),
				EnchantmentKind::Regenerating => {
					if self.health < MAX_HEALTH {
						// Heal every half second
						if effect.frames_left % (30 / effect.enchantment.strength) as u16 == 0 {
							self.health += 1;
						}
					}
				},
			}

			effect.frames_left = effect.frames_left.saturating_sub(1);
			let removing_enchantment = effect.frames_left == 0;

			!removing_enchantment
		});
	}
}

impl AsPolygon for GreenSlime {
	fn as_polygon(&self) -> Polygon {
		let half_size = self.size() * Vec2::splat(0.5);
		easy_polygon(self.pos + half_size, half_size, 0.0)
	}
}

impl Drawable for GreenSlime {
	fn pos(&self) -> Vec2 { self.pos }

	fn size(&self) -> Vec2 { Vec2::splat(SIZE) }

	fn texture(&self) -> Option<Texture2D> { Some(load_my_image("green_slime.webp")) }
}
