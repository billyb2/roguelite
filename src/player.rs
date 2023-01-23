use std::collections::HashMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::attacks::*;
use crate::draw::Drawable;
use crate::enchantments::{Enchantable, Enchantment, EnchantmentKind};
use crate::items::ItemType::{self, *};
use crate::items::{attack_with_item, ItemInfo};
use crate::map::{pos_to_tile, Floor, FloorInfo};
use crate::math::{aabb_collision, easy_polygon, AsPolygon, Polygon};
use macroquad::prelude::*;

pub const PLAYER_SIZE: f32 = 12.0;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlayerClass {
	Warrior,
	Wizard,
	Rogue,
}

impl Display for PlayerClass {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			PlayerClass::Warrior => "Warrior",
			PlayerClass::Wizard => "Wizard",
			PlayerClass::Rogue => "Rogue",
		})
	}
}

pub struct PlayerClassError;

impl TryFrom<&str> for PlayerClass {
	type Error = PlayerClassError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value.to_lowercase().as_str() {
			"warrior" => Ok(PlayerClass::Warrior),
			"wizard" => Ok(PlayerClass::Wizard),
			_ => Err(PlayerClassError),
		}
	}
}

/// Info regarding points such as HP or MP
#[derive(Clone, Debug, Default, Serialize)]
struct PointInfo {
	/// Currently number of points
	points: u16,
	/// The number of frames until your points go up by 1, lower is better
	regen_rate: u16,
	max_points: u16,
	time_til_regen: u16,
}

#[derive(Copy, Clone, Serialize)]
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

#[derive(Clone, Debug, Serialize)]
pub struct ItemSelectedInfo {
	pub index: usize,
	pub selection_type: SelectionType,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum SelectionType {
	Hovered,
	Selected,
}

#[derive(Clone, Serialize)]
pub struct PlayerInventory {
	primary_item: Option<ItemInfo>,
	secondary_item: Option<ItemInfo>,
	selected_item: Option<ItemSelectedInfo>,
	pub items: Vec<ItemInfo>,
}

impl PlayerInventory {
	fn new(primary_item: Option<ItemInfo>, secondary_item: Option<ItemInfo>) -> Self {
		Self {
			primary_item,
			secondary_item,
			selected_item: None,
			items: Vec::new(),
		}
	}

	fn add_item(&mut self, new_item: ItemInfo) {
		if new_item.stack_count.is_some() {
			if let Some(existing_item) = self
				.items
				.iter_mut()
				.chain(
					[&mut self.primary_item, &mut self.secondary_item]
						.into_iter()
						.filter_map(|item| item.as_mut()),
				)
				.find(|item| item.item_type == new_item.item_type)
			{
				existing_item.stack_count = Some(existing_item.stack_count.unwrap() + 1);
			}
		} else {
			self.items.push(new_item);
		}
	}
}

#[derive(Clone, Serialize)]
pub struct Player {
	pub angle: f32,
	pub pos: Vec2,
	speed: f32,
	hp: PointInfo,
	mp: PointInfo,
	/// The ability to resist magical enchantments
	willpower: u16,
	invincibility_frames: u16,

	pub primary_cooldown: u16,
	pub secondary_cooldown: u16,

	spells: Vec<Spell>,

	pub changing_spell: bool,
	pub time_til_change_spell: u8,

	pub xp: u32,
	pub level: u32,

	pub gold: u32,
	in_inventory: bool,
	pub inventory: PlayerInventory,

	enchantments: HashMap<EnchantmentKind, (Enchantment, u16)>,
}

impl Player {
	pub fn new(class: PlayerClass, pos: Vec2) -> Self {
		let primary_item = Some(match class {
			PlayerClass::Warrior => ItemInfo::new(ShortSword, None),
			PlayerClass::Wizard => ItemInfo::new(WizardGlove, None),
			PlayerClass::Rogue => {
				let mut item = ItemInfo::new(ThrowingKnife, None);
				item.stack_count = Some(5);

				item
			},
		});

		let secondary_item = match class {
			PlayerClass::Wizard => Some(ItemInfo::new(WizardsDagger, None)),
			_ => None,
		};

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

			PlayerClass::Rogue => PointInfo {
				points: 20,
				max_points: 20,
				// 15 seconds
				regen_rate: 15 * 60,
				..Default::default()
			},
		};

		let mp = match class {
			PlayerClass::Wizard => PointInfo {
				points: 6,
				max_points: 6,
				// 7 seconds
				regen_rate: 7 * 60,
				..Default::default()
			},
			PlayerClass::Warrior => PointInfo {
				points: 3,
				max_points: 3,
				// 10 seconds
				regen_rate: 10 * 60,
				..Default::default()
			},
			PlayerClass::Rogue => PointInfo {
				points: 4,
				max_points: 4,
				regen_rate: 9 * 60,
				..Default::default()
			},
		};

		let willpower = match class {
			PlayerClass::Wizard => 20,
			PlayerClass::Warrior => 10,
			PlayerClass::Rogue => 15,
		};

		let spells = match class {
			PlayerClass::Warrior => Vec::new(),
			PlayerClass::Rogue => Vec::new(),
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
			spells,
			changing_spell: false,
			time_til_change_spell: 0,
			xp: 0,
			level: 0,
			gold: 0,
			in_inventory: false,
			inventory: PlayerInventory::new(primary_item, secondary_item),
			enchantments: HashMap::new(),
		}
	}

	pub fn add_xp(&mut self, xp: u32) {
		self.xp += xp;

		let xp_to_level_up = match self.level {
			0 => 14,
			1 => 16,
			_ => todo!(),
		};

		if self.xp >= xp_to_level_up {
			self.xp = 0;
			self.level += 1;

			self.mp.max_points += 2;
			self.mp.points += 2;

			self.hp.max_points += 1;
			self.hp.points += 1;

			println!("Leveled up!");
		}
	}

	pub fn inventory(&self) -> &PlayerInventory { &self.inventory }

	pub fn set_selected_item(&mut self, i: Option<ItemSelectedInfo>) {
		self.inventory.selected_item = i;
	}

	pub fn get_item_selection_type(&self) -> Option<&SelectionType> {
		self.inventory
			.selected_item
			.as_ref()
			.map(|item_selected_info| &item_selected_info.selection_type)
	}

	/// Replace the first and last items in the spells Vec
	pub fn cycle_spells(&mut self) {
		let spells_len = self.spells.len();
		self.spells.swap(0, spells_len - 1);
	}

	#[inline]
	pub fn pos(&self) -> Vec2 { self.pos }

	#[inline]
	pub fn hp(&self) -> u16 { self.hp.points }

	#[inline]
	pub fn mp(&self) -> u16 { self.mp.points }

	#[inline]
	pub fn spells(&self) -> &[Spell] { &self.spells }

	#[inline]
	pub fn enchantments(&self) -> &HashMap<EnchantmentKind, (Enchantment, u16)> {
		&self.enchantments
	}
}

pub fn move_player(player: &mut Player, angle: f32, speed: Option<Vec2>, floor_info: &Floor) {
	let direction: Vec2 = (angle.cos(), angle.sin()).into();
	let distance = direction *
		speed.unwrap_or_else(|| {
			let speed_mul = match player.enchantments.get(&EnchantmentKind::Sticky) {
				Some((enchantnment, _)) => 1.0 / enchantnment.strength as f32,
				None => 1.0,
			};

			let speed = player.speed * speed_mul;
			Vec2::splat(speed)
		});

	let collision_info = floor_info.collision_dir(player, distance);

	if !collision_info.x {
		player.pos.x += distance.x;
	}

	if !collision_info.y {
		player.pos.y += distance.y;
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
			}
		}
	};

	players.iter_mut().for_each(|player| {
		if player.hp.points != 0 {
			player.primary_cooldown = player.primary_cooldown.saturating_sub(1);
			player.secondary_cooldown = player.secondary_cooldown.saturating_sub(1);

			player.invincibility_frames = player.invincibility_frames.saturating_sub(1);

			player.time_til_change_spell = player.time_til_change_spell.saturating_sub(1);

			if player.changing_spell &&
				player.time_til_change_spell == 0 &&
				!player.spells.is_empty()
			{
				player.cycle_spells();
				player.changing_spell = false;
			}

			regen(&mut player.hp);
			regen(&mut player.mp);
		}
	});
}

pub fn player_attack(
	player: &mut Player, index: Option<usize>, attacks: &mut Vec<AttackObj>, floor: &FloorInfo,
	is_primary: bool,
) {
	let cooldown = match is_primary {
		true => &player.primary_cooldown,
		false => &player.secondary_cooldown,
	};

	if *cooldown != 0 {
		return;
	}

	let item = match is_primary {
		true => &mut player.inventory.primary_item,
		false => &mut player.inventory.secondary_item,
	};

	if let Some(item) = item {
		if item.item_type == ItemType::ThrowingKnife {
			if item.stack_count.unwrap() > 0 {
				item.stack_count = Some(item.stack_count.unwrap() - 1);
			} else {
				return;
			}
		}

		if let Some(attack) = attack_with_item(item.clone(), player, index, floor, is_primary) {
			let cooldown = match is_primary {
				true => &mut player.primary_cooldown,
				false => &mut player.secondary_cooldown,
			};

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

pub struct DamageInfo {
	pub damage: u16,
	pub direction: f32,
	pub player: usize,
}

pub enum DoorInteraction {
	Opening,
	Closing,
	Toggle,
}

pub fn interact_with_door<A: AsPolygon>(
	entity: &A, door_interaction: DoorInteraction, floor_info: &mut FloorInfo,
) {
	// First, see if the player is in contact with a door
	let entity_tile_pos = pos_to_tile(entity);

	// Find all door that's within one tile distance of the player, then pick the
	// closest one

	let door = floor_info
		.floor
		.doors()
		.filter(|door| {
			let tile_distance = (door.tile_pos() - entity_tile_pos).abs();

			let entity_in_door = floor_info
				.monsters
				.iter()
				.map(|m| pos_to_tile(&m.as_polygon()))
				.any(|pos| pos == door.tile_pos());

			// You can't open or close doors that you're inside of
			tile_distance.cmple(IVec2::ONE).all() &&
				!door.tile_pos().eq(&entity_tile_pos) &&
				!entity_in_door
		})
		.reduce(|door_obj, door2_obj| {
			let door = &door_obj.door().unwrap();
			let door2 = &door2_obj.door().unwrap();

			// First, depending on the action the player is taking, we can pretty easily
			// decide of the player wants to open or close the door
			let door_will_be_affected = match door_interaction {
				DoorInteraction::Opening => !door.is_open,
				DoorInteraction::Closing => door.is_open,
				DoorInteraction::Toggle => true,
			};

			let door2_will_be_affected = match door_interaction {
				DoorInteraction::Opening => !door2.is_open,
				DoorInteraction::Closing => door2.is_open,
				DoorInteraction::Toggle => true,
			};

			if door_will_be_affected && door2_will_be_affected {
				// Check which door the plyer is touching
				if aabb_collision(door_obj, entity, Vec2::ZERO) {
					door_obj
				} else {
					door2_obj
				}
			} else if door_will_be_affected {
				door_obj
			} else {
				door2_obj
			}
		});

	if let Some(door_obj) = door {
		match door_interaction {
			DoorInteraction::Opening => door_obj.open_door(),
			DoorInteraction::Closing => door_obj.close_door(),
			DoorInteraction::Toggle => match door_obj.door().unwrap().is_open {
				true => door_obj.close_door(),
				false => door_obj.open_door(),
			},
		};
	}
}

impl AsPolygon for Player {
	fn as_polygon(&self) -> Polygon {
		const HALF_SIZE: Vec2 = Vec2::splat(PLAYER_SIZE * 0.5);
		easy_polygon(self.pos + HALF_SIZE, HALF_SIZE, 0.0)
	}
}

impl Drawable for Player {
	fn pos(&self) -> Vec2 { self.pos }

	fn size(&self) -> Vec2 { Vec2::splat(PLAYER_SIZE) }

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
		if self.enchantments.get(&enchantment.kind).is_none() {
			let enchantment_time = match enchantment.kind {
				EnchantmentKind::Blinded => 60,
				EnchantmentKind::Sticky => 60,
				EnchantmentKind::Regenerating => 60 * 8,
			};

			self.enchantments
				.insert(enchantment.kind, (enchantment, enchantment_time));
		}
	}

	fn update_enchantments(&mut self) {
		self.enchantments
			.retain(|enchantment_kind, (enchantment, time_til_removal)| {
				// Regenerates the player's health every second
				if *enchantment_kind == EnchantmentKind::Regenerating {
					if *time_til_removal % (60 / enchantment.strength as u16) == 0 {
						if self.hp.points < self.hp.max_points {
							self.hp.points += 1;
						}
					}
				}

				*time_til_removal -= 1;
				*time_til_removal != 0
			});
	}
}

pub fn pickup_items(player: &mut Player, floor: &mut Floor) {
	let mut item = None;

	'search: for i in 0..floor.objects().len() {
		let object = &mut floor.objects_mut()[i];
		let items = object.items_mut();

		for i in 0..items.len() {
			if aabb_collision(&items[i], player, Vec2::ZERO) {
				item = Some(items.remove(i));
				break 'search;
			}
		}
	}

	if let Some(item) = item {
		match item.item_type {
			ItemType::Gold(gold) => player.gold += gold,
			_ => player.inventory.add_item(item),
		};
	}
}

pub fn toggle_inventory(player: &mut Player) { player.in_inventory = !player.in_inventory; }

pub const ITEM_INVENTORY_SIZE: Vec2 = Vec2::splat(50.0);

pub fn item_pos_from_index(i: usize) -> Vec2 {
	Vec2::new(100.0, 150.0) +
		ITEM_INVENTORY_SIZE +
		(UVec2::new(i as u32 % 10, i as u32 / 10) * ITEM_INVENTORY_SIZE.as_uvec2()).as_vec2()
}

pub fn draw_inventory(player: &Player) {
	if !player.in_inventory {
		return;
	}

	draw_rectangle(100.0, 100.0, 650.0, 450.0, LIGHTGRAY);
	draw_rectangle_lines(100.0, 100.0, 650.0, 450.0, 15.0, DARKGRAY);

	player
		.inventory
		.items
		.iter()
		.enumerate()
		.for_each(|(i, item)| {
			let texture = item.texture().unwrap();

			let texture_params = DrawTextureParams {
				rotation: item.rotation(),
				flip_x: item.flip_x(),
				dest_size: Some(ITEM_INVENTORY_SIZE),
				..Default::default()
			};

			let item_pos = item_pos_from_index(i);

			let color = match player
				.inventory
				.selected_item
				.as_ref()
				.map(|info| info.index) ==
				Some(i)
			{
				true => RED,
				false => DARKGRAY,
			};

			draw_rectangle_lines(
				item_pos.x,
				item_pos.y,
				ITEM_INVENTORY_SIZE.x,
				ITEM_INVENTORY_SIZE.y,
				8.0,
				color,
			);

			draw_texture_ex(texture, item_pos.x, item_pos.y, WHITE, texture_params);
		});
}
