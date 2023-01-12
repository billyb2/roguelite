use std::collections::{HashMap, HashSet};

use crate::draw::{load_my_image, Drawable};
use crate::enchantments::{Enchantable, Enchantment, EnchantmentKind};
use crate::map::{pos_to_tile, Floor, Object, TILE_SIZE};
use crate::math::{aabb_collision, easy_polygon, get_angle, AsPolygon, Polygon};
use crate::monsters::Monster;
use crate::player::{damage_player, DamageInfo, Player};

use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;

use super::Effect;

#[derive(PartialEq)]
enum AttackMode {
	Passive,
	Attacking,
}

#[derive(Copy, Clone)]
enum Target {
	Pos(Vec2),
	PlayerIndex(usize),
}

const SIZE: f32 = 18.0;
const MAX_HEALTH: u16 = 22;

pub struct SmallRat {
	health: u16,
	pos: Vec2,
	speed_mul: f32,
	texture: Texture2D,
	attack_mode: AttackMode,
	time_spent_moving: u16,
	time_til_move: u16,
	current_path: Option<(Vec<Vec2>, usize)>,
	enchantments: HashMap<EnchantmentKind, Effect>,
	// All the players who have damaged me
	damaged_by: HashSet<usize>,
	// Gotta keep track of if the target moved, to reset the path
	current_target: Option<Target>,
}

impl Monster for SmallRat {
	fn new(pos: Vec2) -> Box<dyn Monster> {
		let monster: Box<dyn Monster> = Box::new(Self {
			pos,
			health: MAX_HEALTH,
			texture: load_my_image("small_mouse.webp"),
			attack_mode: AttackMode::Passive,
			time_til_move: rand::gen_range(0_u32, 180).try_into().unwrap(),
			time_spent_moving: 0,
			current_path: None,
			current_target: None,
			enchantments: HashMap::new(),
			damaged_by: HashSet::new(),
			speed_mul: 1.0,
		});

		monster
	}

	fn movement(&mut self, players: &[Player], floor: &Floor) {
		if self.enchantments.contains_key(&EnchantmentKind::Blinded) {
			move_blindly(self, floor);
		} else {
			match self.attack_mode {
				AttackMode::Passive => passive_mode(self, players, floor),
				AttackMode::Attacking => attack_mode(self, players, floor),
			};
		}
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

	fn take_damage(&mut self, damage_info: DamageInfo, floor: &Floor) {
		self.health = self.health.saturating_sub(damage_info.damage);

		self.enchantments.iter_mut().for_each(|enchantment| {
			enchantment.1.frames_left /= 2;
		});

		let change = Vec2::new(damage_info.direction.cos(), damage_info.direction.sin()) *
			Vec2::splat(SIZE) *
			Vec2::splat((damage_info.damage as f32 / MAX_HEALTH as f32).clamp(0.0, 0.8));

		if !floor.collision(self, change) {
			self.pos += change;
		}

		self.damaged_by.insert(damage_info.player);
	}

	fn living(&self) -> bool { self.health > 0 }

	fn xp(&self) -> (&HashSet<usize>, u32) {
		const DEFAULT_XP: u32 = 1;
		// Divide the XP between all players
		(&self.damaged_by, DEFAULT_XP)
	}
}

fn player_in_aggro_range((_, player): &(usize, &Player), visible_objects: &[&Object]) -> bool {
	if player.hp() == 0 {
		return false;
	}

	let player_tile_pos = pos_to_tile(*player);

	visible_objects
		.iter()
		.any(|o| o.tile_pos() == player_tile_pos)
}

fn step_pathfinding<T: Fn(&mut SmallRat) -> Target>(
	my_monster: &mut SmallRat, players: &[Player], floor: &Floor, speed: f32, find_target: T,
) {
	if my_monster.time_til_move == 0 {
		if my_monster.current_path.is_none() {
			if let Some(target) = my_monster.current_target {
				let goal_aabb: Polygon = match target {
					Target::Pos(pos) => {
						const HALF_TILE_SIZE: Vec2 = Vec2::splat((TILE_SIZE / 2) as f32);
						easy_polygon(pos + HALF_TILE_SIZE, HALF_TILE_SIZE, 0.0)
					},
					Target::PlayerIndex(i) => {
						let player = &players[i];
						player.as_polygon()
					},
				};

				if let Some(path) = floor.find_path(my_monster, &goal_aabb, true, false, Some(4)) {
					my_monster.current_path = Some((path, 1));
				} else {
					my_monster.current_target = Some(find_target(my_monster));
					return;
				}
			} else {
				my_monster.current_target = Some(find_target(my_monster));
				return;
			}
		}

		if let Some((path, i)) = &mut my_monster.current_path {
			if let Some(pos) = path.get(*i) {
				let distance_to_target = my_monster.pos.distance(*pos);

				if speed >= distance_to_target {
					my_monster.pos = *pos;
					*i += 1;

					// On every even turn
					if *i & 1 == 0 {
						// If our target moves, then change our path
						let target_pos = match my_monster.current_target.unwrap() {
							Target::Pos(pos) => pos,
							Target::PlayerIndex(i) => players[i].pos,
						};

						if *pos != target_pos {
							my_monster.current_path = None;
						}
					}
				} else {
					let angle = get_angle(*pos, my_monster.pos);
					let change = Vec2::new(angle.cos(), angle.sin()) * speed * my_monster.speed_mul;

					if floor.collision(my_monster, change) {
						// Only stop targetting when there is a door
						my_monster.current_path = None;
						my_monster.current_target = None;
					} else {
						my_monster.pos += change;
					}
				}
			} else {
				// Finished following path
				my_monster.current_path = None;
				my_monster.current_target = None;
				my_monster.time_til_move = 0;
			}
		}
	}
}

// The rat just wanders around a lil in passive mode
fn passive_mode(my_monster: &mut SmallRat, players: &[Player], floor: &Floor) {
	my_monster.time_til_move = my_monster.time_til_move.saturating_sub(1);

	if my_monster.time_til_move > 0 {
		return;
	}

	let visible_objects = floor.visible_objects(my_monster, Some(8));

	let find_target = |_my_monster: &mut SmallRat| -> Target {
		// Choose a random visible tile
		let target_obj = visible_objects.choose().unwrap();
		Target::Pos(target_obj.pos())
	};

	if my_monster.current_target.is_none() {
		my_monster.current_target = Some(find_target(my_monster));
		my_monster.current_path = None;
	}

	step_pathfinding(my_monster, players, floor, 0.75, find_target);

	// If a player is visible to the rat, attack them
	if let Some((i, _)) = players
		.iter()
		.enumerate()
		.find(|p_info| player_in_aggro_range(p_info, &visible_objects))
	{
		my_monster.time_til_move = 25;
		my_monster.time_spent_moving = 0;

		my_monster.attack_mode = AttackMode::Attacking;
		my_monster.current_target = Some(Target::PlayerIndex(i));
		my_monster.current_path = None;
	}
}

fn attack_mode(my_monster: &mut SmallRat, players: &[Player], floor: &Floor) {
	my_monster.time_til_move = my_monster.time_til_move.saturating_sub(1);

	if my_monster.time_til_move > 0 {
		return;
	}

	let find_target = |my_monster: &mut SmallRat| {
		match my_monster.current_target {
			Some(target) => target,
			None => {
				let visible_objects = floor.visible_objects(my_monster, Some(8));

				let player_index: Option<usize> =
					players.iter().enumerate().find_map(|(i, player)| {
						let p_tile_pos = pos_to_tile(player);
						let player_is_visible = visible_objects
							.iter()
							.any(|v_obj| v_obj.tile_pos() == p_tile_pos);

						match player_is_visible {
							true => Some(i),
							false => None,
						}
					});

				match player_index {
					Some(index) => Target::PlayerIndex(index),
					None => {
						// If there are no visible players, then just go back to passive mode
						my_monster.attack_mode = AttackMode::Passive;
						Target::Pos(my_monster.center())
					},
				}
			},
		}
	};

	step_pathfinding(my_monster, players, floor, 1.1, find_target);

	if let Some(Target::PlayerIndex(i)) = my_monster.current_target {
		let target_player = &players[i];

		let distance_from_target = target_player.center().distance(my_monster.center());

		// When the monster's within range of the player, "lunge" at them
		if distance_from_target <= TILE_SIZE as f32 {
			let angle = get_angle(target_player.pos(), my_monster.pos);
			my_monster.pos += Vec2::new(angle.cos(), angle.sin()) * SIZE;
			my_monster.time_til_move = 45;
			my_monster.current_path = None;
		}
		// If the player dies, go back to passive mode
		if target_player.hp() == 0 {
			my_monster.attack_mode = AttackMode::Passive;
			my_monster.current_target = None;
		}
	}
}

fn move_blindly(my_monster: &mut SmallRat, floor: &Floor) {
	if my_monster.time_til_move > 0 {
		my_monster.time_til_move = my_monster.time_til_move.saturating_sub(1);
		return;
	}

	if let Some(Target::Pos(pos)) = my_monster.current_target {
		if pos.distance(my_monster.pos) < SIZE as f32 {
			my_monster.current_target = None;
		}

		let angle = get_angle(pos, my_monster.pos);
		let change = Vec2::new(angle.cos(), angle.sin()) * Vec2::splat(1.2) * my_monster.speed_mul;

		if !floor.collision(my_monster, change) {
			my_monster.pos += change;
		} else {
			let change = change * 1.5;
			if !floor.collision(my_monster, -change) {
				my_monster.pos -= change;
			}
			my_monster.current_target = None;
			my_monster.time_til_move = 30;
		}
	} else {
		let direction = Vec2::new(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0));

		my_monster.current_target = Some(Target::Pos(
			direction * Vec2::splat((TILE_SIZE * 2) as f32) +
				my_monster.pos + Vec2::splat(SIZE * 0.25),
		));
	}
}

impl Enchantable for SmallRat {
	fn apply_enchantment(&mut self, enchantment: Enchantment) {
		match enchantment.kind {
			EnchantmentKind::Blinded => {
				self.current_target = None;
				self.current_path = None;
				self.time_til_move = 50;
			},
			EnchantmentKind::Sticky => {
				self.speed_mul = 0.5;
			},
			EnchantmentKind::Regenerating => (),
		};

		self.enchantments.insert(
			enchantment.kind,
			Effect {
				frames_left: 240,
				enchantment,
			},
		);
	}

	fn update_enchantments(&mut self) {
		self.enchantments.retain(|e_kind, effect| {
			match e_kind {
				EnchantmentKind::Blinded => (),
				EnchantmentKind::Sticky => (),
				EnchantmentKind::Regenerating => {
					if self.health < MAX_HEALTH {
						// Heal every half second
						if effect.frames_left % (30 / effect.enchantment.strength) as u16 == 0 {
							self.health += 1;
						}
					}
				},
			};

			effect.frames_left = effect.frames_left.saturating_sub(1);
			let removing_enchantment = effect.frames_left == 0;

			if removing_enchantment {
				match e_kind {
					EnchantmentKind::Blinded => {
						self.attack_mode = AttackMode::Passive;
						self.time_til_move = 10;
						self.time_spent_moving = 0;
						self.current_target = None;
						self.current_path = None;
					},
					EnchantmentKind::Sticky => {
						self.speed_mul = 1.0;
					},
					EnchantmentKind::Regenerating => (),
				}
			}

			!removing_enchantment
		});
	}
}

impl AsPolygon for SmallRat {
	fn as_polygon(&self) -> Polygon {
		const HALF_SIZE: Vec2 = Vec2::splat(SIZE * 0.5);
		easy_polygon(self.pos + HALF_SIZE, HALF_SIZE, 0.0)
	}
}

impl Drawable for SmallRat {
	fn pos(&self) -> Vec2 { self.pos }

	fn size(&self) -> Vec2 {
		match self.attack_mode {
			AttackMode::Attacking => Vec2::splat(SIZE * 1.1),
			_ => Vec2::splat(SIZE),
		}
	}

	fn flip_x(&self) -> bool { true }

	fn texture(&self) -> Option<Texture2D> { Some(self.texture) }
}
