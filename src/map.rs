use std::collections::HashMap;

use macroquad::prelude::*;
use macroquad::rand;
use macroquad::rand::*;
use pathfinding::prelude::*;
#[cfg(feature = "native")]
use rayon::prelude::*;
use serde::Serialize;

use crate::draw::{load_my_image, Drawable};
use crate::enchantments::{Enchantable, Enchantment, EnchantmentKind};
use crate::items::{ItemInfo, ItemType, PotionType};
use crate::math::{
	aabb_collision,
	aabb_collision_dir,
	easy_polygon,
	points_on_circumference,
	points_on_line,
	AsPolygon,
	Polygon,
};
use crate::monsters::{GreenSlime, Monster, MonsterObj, SmallRat};
use crate::player::Player;

pub const TILE_SIZE: usize = 30;

pub const MAP_WIDTH_TILES: usize = 50;
pub const MAP_HEIGHT_TILES: usize = 50;

pub const MAP_SIZE_TILES: IVec2 = IVec2::new(MAP_WIDTH_TILES as i32, MAP_HEIGHT_TILES as i32);

#[derive(Copy, Clone, Debug, Serialize)]
enum TrapType {
	Teleport,
	SpawnMonster,
}

#[derive(Copy, Clone, Debug, Serialize)]
struct Trap {
	triggered: bool,
	trap_type: TrapType,
}

#[derive(Copy, Clone, Debug, Serialize)]
enum EffectType {
	Slimed,
}

#[derive(Clone, Debug, Serialize)]
struct Effect {
	time_til_dissipate: Option<u16>,
	effect_type: EffectType,
}

impl Into<Enchantment> for EffectType {
	fn into(self) -> Enchantment {
		Enchantment {
			strength: 1,
			kind: EnchantmentKind::Sticky,
		}
	}
}

#[derive(Clone, Debug, Serialize)]
pub struct Object {
	pos: IVec2,
	is_floor: bool,
	has_been_seen: bool,
	is_currently_visible: bool,
	items: Vec<ItemInfo>,
	door: Option<Door>,
	trap: Option<Trap>,
	effects: HashMap<EffectType, Effect>,
}

impl Default for Object {
	fn default() -> Self {
		Self {
			pos: IVec2::ZERO,
			is_floor: false,
			is_currently_visible: false,
			has_been_seen: false,
			items: Vec::new(),
			door: None,
			trap: None,
			effects: HashMap::new(),
		}
	}
}

impl PartialEq for Object {
	fn eq(&self, other: &Self) -> bool { self.pos == other.pos }
}

impl Object {
	pub fn tile_pos(&self) -> IVec2 { self.pos }

	pub fn is_collidable(&self) -> bool {
		if self.is_floor {
			return false;
		}

		match &self.door {
			Some(door) => !door.is_open,
			None => true,
		}
	}

	pub fn items(&self) -> &[ItemInfo] { &self.items }

	pub fn door(&self) -> &Option<Door> { &self.door }

	pub fn has_been_seen(&self) -> bool { self.has_been_seen }

	pub fn items_mut(&mut self) -> &mut Vec<ItemInfo> { &mut self.items }

	pub fn open_door(&mut self) {
		if let Some(door) = &mut self.door {
			door.open();
		}
	}

	pub fn close_door(&mut self) {
		if let Some(door) = &mut self.door {
			door.close();
		}
	}

	pub fn clear_currently_visible(&mut self) { self.is_currently_visible = false; }

	pub fn currently_visible(&self) -> bool { self.is_currently_visible }
}

impl AsPolygon for Object {
	fn as_polygon(&self) -> Polygon {
		const HALF_TILE_SIZE: Vec2 = Vec2::splat(TILE_SIZE as f32 / 2.0);
		easy_polygon(
			(self.pos.as_vec2() * Vec2::splat(TILE_SIZE as f32)) + HALF_TILE_SIZE,
			HALF_TILE_SIZE,
			0.0,
		)
	}
}

impl AsPolygon for &Object {
	fn as_polygon(&self) -> Polygon { (*self).as_polygon() }
}

#[derive(Copy, Clone, Debug, Serialize)]
pub struct Door {
	pos: IVec2,
	pub is_open: bool,
}

impl Door {
	pub fn open(&mut self) { self.is_open = true; }

	pub fn close(&mut self) { self.is_open = false; }
}

#[derive(Clone, Serialize)]
pub struct Room {
	top_left: IVec2,
	bottom_right: IVec2,
	doors: Vec<Door>,
}

impl Room {
	pub fn extents(&self) -> (IVec2, IVec2) { (self.top_left, self.bottom_right) }

	fn generate_walls(&self) -> Vec<IVec2> {
		(self.top_left.x..self.bottom_right.x)
			.into_iter()
			.flat_map(|x| {
				[
					IVec2::new(x, self.top_left.y),
					IVec2::new(x, self.bottom_right.y),
				]
				.into_iter()
			})
			.chain(
				(self.top_left.y..=self.bottom_right.y)
					.into_iter()
					.flat_map(|y| {
						[
							IVec2::new(self.top_left.x, y),
							IVec2::new(self.bottom_right.x, y),
						]
						.into_iter()
					}),
			)
			.collect()
	}

	fn generate_wall_objects(&self) -> Vec<Object> {
		self.generate_walls()
			.into_iter()
			.map(|w_pos| {
				let door = self.doors.iter().find(|d| d.pos == w_pos).copied();

				Object {
					pos: w_pos,
					is_floor: false,
					trap: None,
					has_been_seen: false,
					items: Vec::new(),
					door,
					..Default::default()
				}
			})
			.collect()
	}

	fn generate_floor(&self) -> Vec<Object> {
		let map_object = |x: i32, y: i32| -> Object {
			// 1 in 250 chance of being a trapped tile
			let is_trap: bool = rand::gen_range(1, 250) == 100;

			let trap = match is_trap {
				true => Some(Trap {
					triggered: false,
					trap_type: match rand::rand() > u32::MAX / 2 {
						true => TrapType::Teleport,
						false => TrapType::SpawnMonster,
					},
				}),
				false => None,
			};

			let _texture = match is_trap {
				true => load_my_image("trap.webp"),
				false => load_my_image("light_gray.webp"),
			};

			// 1 in every 100 tiles have a 1 in 10 chance of having gold
			let mut items = Vec::new();

			let pos = IVec2::new(x, y);

			if rand::gen_range(0, 50) == 25 {
				items.push(ItemInfo::new(
					ItemType::Potion(PotionType::Regeneration),
					Some(pos),
				));
			}

			Object {
				pos,
				is_floor: true,
				has_been_seen: false,
				door: None,
				items,
				trap,
				..Default::default()
			}
		};

		((self.top_left.x..self.bottom_right.x)
			.into_iter()
			.flat_map(|x| {
				(self.top_left.y..self.bottom_right.y)
					.into_iter()
					.map(move |y| map_object(x, y))
			}))
		.collect()
	}

	/// Returns whether or not a position is inside a room
	fn inside_room(&self, pos: IVec2) -> bool {
		pos.cmpgt(self.top_left).all() && pos.cmplt(self.bottom_right).all()
	}

	pub fn center(&self) -> IVec2 { (self.top_left + self.bottom_right) / 2 }
}

#[derive(Clone, Serialize)]
pub struct FloorInfo {
	spawn: Vec2,
	monster_types: Vec<MonsterObj>,
	item_types: Vec<ItemType>,
	pub monsters: Vec<MonsterObj>,
	pub floor: Floor,
	rooms: Vec<Room>,
	exit: Object,
}

impl FloorInfo {
	pub fn new(_floor_num: usize) -> Self {
		let mut rooms = Vec::new();

		// First, try to flll the map with as many rooms as possible
		for _ in 0..100_000 {
			const MIN_SIZE: i32 = 8;
			const MAX_SIZE: i32 = 14;

			let top_left = IVec2::new(
				rand::gen_range(0, MAP_WIDTH_TILES as i32),
				rand::gen_range(0, MAP_HEIGHT_TILES as i32),
			);
			let bottom_right = top_left +
				IVec2::new(
					rand::gen_range(MIN_SIZE, MAX_SIZE),
					rand::gen_range(MIN_SIZE, MAX_SIZE),
				);

			// Fail if the map extends past the map border
			if bottom_right
				.cmpgt(IVec2::new(MAP_WIDTH_TILES as i32, MAP_HEIGHT_TILES as i32))
				.any()
			{
				continue;
			}

			if !rooms.iter().any(|room: &Room| {
				// Don't let rooms be just one room apart, since moving through doors fro both
				// rooms is annoying to the player Also don't let rooms collide w each other
				const MIN_DISTANCE_BETWEEN_ROOMS: i32 = 2;

				(room.bottom_right + MIN_DISTANCE_BETWEEN_ROOMS)
					.cmpgt(top_left - MIN_DISTANCE_BETWEEN_ROOMS)
					.all() && (bottom_right + MIN_DISTANCE_BETWEEN_ROOMS)
					.cmpgt(room.top_left - MIN_DISTANCE_BETWEEN_ROOMS)
					.all()
			}) {
				rooms.push(Room {
					top_left,
					bottom_right,
					doors: Vec::new(),
				})
			}
		}

		// Then, remove rooms until we have the number of rooms we actually wanted
		rooms.shuffle();
		// rooms.drain(0..(rooms.len() - MAX_NUM_ROOMS));
		// assert!(rooms.len() == MAX_NUM_ROOMS);

		let mut hallways: Vec<IVec2> = rooms //[0..(rooms.len() * 3) / 4]
			.iter()
			.flat_map(|room: &Room| {
				let (top_left_room, bottom_right_room) = (room.top_left, room.bottom_right);

				let other_room = rooms.choose().unwrap();
				let (top_left_other_room, bottom_right_other_room) =
					(other_room.top_left, other_room.bottom_right);

				let room_center = (top_left_room + bottom_right_room) / 2;
				let other_room_center = (top_left_other_room + bottom_right_other_room) / 2;

				let (mut left_room, mut right_room) = match room_center.x < other_room_center.x {
					true => (room_center, other_room_center),
					false => (other_room_center, room_center),
				};

				let center_touching_wall = |room: &Room, center_ivec4: IVec4| -> bool {
					let room_extremes = IVec4::new(
						room.top_left.x,
						room.top_left.y,
						room.bottom_right.x,
						room.bottom_right.y,
					);
					center_ivec4.cmpeq(room_extremes).any()
				};

				let left_room_ivec4 =
					IVec4::new(left_room.x, left_room.y, left_room.x, left_room.y);

				let right_room_ivec4 =
					IVec4::new(right_room.x, right_room.y, right_room.x, right_room.y);

				if rooms
					.iter()
					.any(|room| center_touching_wall(room, left_room_ivec4))
				{
					let new_left_room = (-2..=2).into_iter().find_map(|change| {
						let left_room_x_change = left_room + IVec2::new(change, 0);
						let left_room_y_change = left_room + IVec2::new(0, change);

						let left_room_x_change_ivec4 = IVec4::new(
							left_room_x_change.x,
							left_room_x_change.y,
							left_room_x_change.x,
							left_room_x_change.y,
						);

						let left_room_y_change_ivec4 = IVec4::new(
							left_room_y_change.x,
							left_room_y_change.y,
							left_room_y_change.x,
							left_room_y_change.y,
						);

						rooms.iter().find_map(|room_extremes| {
							if !center_touching_wall(room_extremes, left_room_x_change_ivec4) {
								Some(left_room_x_change)
							} else if !center_touching_wall(room_extremes, left_room_y_change_ivec4)
							{
								Some(left_room_y_change)
							} else {
								None
							}
						})
					});

					if let Some(new_left_room) = new_left_room {
						left_room = new_left_room;
					} else {
						return None;
					}
				}

				if rooms
					.iter()
					.any(|room_extremes| center_touching_wall(room_extremes, right_room_ivec4))
				{
					let new_right_room = (-2..=2).into_iter().find_map(|change| {
						let right_room_x_change = right_room + IVec2::new(change, 0);
						let right_room_y_change = right_room + IVec2::new(0, change);

						let right_room_x_change_ivec4 = IVec4::new(
							right_room_x_change.x,
							right_room_x_change.y,
							right_room_x_change.x,
							right_room_x_change.y,
						);

						let right_room_y_change_ivec4 = IVec4::new(
							right_room_y_change.x,
							right_room_y_change.y,
							right_room_y_change.x,
							right_room_y_change.y,
						);

						rooms.iter().find_map(|room_extremes| {
							if !center_touching_wall(room_extremes, right_room_x_change_ivec4) {
								Some(right_room_x_change)
							} else if !center_touching_wall(
								room_extremes,
								right_room_y_change_ivec4,
							) {
								Some(right_room_y_change)
							} else {
								None
							}
						})
					});

					if let Some(new_right_room) = new_right_room {
						right_room = new_right_room;
					} else {
						return None;
					}
				}
				Some(
					(left_room.x..=right_room.x)
						.into_iter()
						.map(move |x| IVec2::new(x, left_room.y))
						.chain(
							((left_room.y.min(right_room.y) - 1)..=left_room.y.max(right_room.y))
								.into_iter()
								.map(move |y| IVec2::new(right_room.x, y)),
						),
				)
			})
			.flatten()
			.collect();

		rooms.iter_mut().for_each(|room| {
			let room_walls = room.generate_walls();

			room_walls
				.iter()
				.filter(|w| hallways.iter().any(|h| h == *w))
				.for_each(|&door_pos| {
					// Fix a bug where doors can pop up in the corners of rooms
					if door_pos != room.top_left &&
						door_pos != room.bottom_right &&
						door_pos != IVec2::new(room.bottom_right.x, room.top_left.y) &&
						door_pos != IVec2::new(room.top_left.x, room.bottom_right.y)
					{
						room.doors.push(Door {
							pos: door_pos,
							is_open: false,
						});
					}
				});
		});

		// Only keep rooms that have a door
		rooms.retain(|room| !room.doors.is_empty());

		// Remove all "hallway" positions inside of a room
		hallways.retain(|h| !rooms.iter().any(|r| r.inside_room(*h)));

		// Actually render all of the walls
		let walls = (0..MAP_WIDTH_TILES as i32).into_iter().flat_map(|x| {
			{
				[
					Object {
						pos: IVec2::new(x, 0),
						door: None,
						has_been_seen: false,
						is_floor: false,
						items: Vec::new(),
						trap: None,
						..Default::default()
					},
					Object {
						pos: IVec2::new(x, MAP_HEIGHT_TILES as i32),
						door: None,
						has_been_seen: false,
						is_floor: false,
						items: Vec::new(),
						trap: None,

						..Default::default()
					},
				]
				.into_iter()
			}
			.chain((0..=MAP_HEIGHT_TILES as i32).into_iter().flat_map(|y| {
				[
					Object {
						pos: IVec2::new(0, y),
						door: None,
						has_been_seen: false,
						is_floor: false,
						items: Vec::new(),
						trap: None,

						..Default::default()
					},
					Object {
						pos: IVec2::new(MAP_WIDTH_TILES as i32, y),
						door: None,
						has_been_seen: false,
						is_floor: false,
						items: Vec::new(),
						trap: None,

						..Default::default()
					},
				]
				.into_iter()
			}))
		});

		let room_walls = rooms
			.iter()
			.flat_map(|room: &Room| room.generate_wall_objects());

		let rooms_ref = &rooms;
		let hallways_ref = &hallways;

		let dungeon_walls = (0..MAP_WIDTH_TILES).into_iter().flat_map(|x| {
			(0..MAP_HEIGHT_TILES).into_iter().filter_map(move |y| {
				let pos = IVec2::new(x as i32, y as i32);
				let in_room = rooms_ref.iter().any(|r| r.inside_room(pos));
				let is_hallway = hallways_ref.iter().any(|h| *h == pos);

				let is_dungeon_wall = !in_room && !is_hallway;

				if is_dungeon_wall {
					Some(Object {
						pos,
						is_floor: false,
						has_been_seen: false,
						items: Vec::new(),
						door: None,
						trap: None,

						..Default::default()
					})
				} else {
					None
				}
			})
		});

		let collidable_objects: Vec<Object> =
			walls.chain(room_walls).chain(dungeon_walls).collect();

		let background_objects: Vec<Object> = hallways
			.iter()
			.map(|&pos| Object {
				pos,
				door: None,
				has_been_seen: false,
				items: Vec::new(),
				trap: None,
				is_floor: true,
				..Default::default()
			})
			.chain(rooms.iter().flat_map(|r| r.generate_floor()))
			.collect();

		let spawn = rooms
			.choose()
			.map(|r| {
				(((r.top_left + r.bottom_right) / 2) * IVec2::splat(TILE_SIZE as i32)).as_vec2()
			})
			.unwrap();

		let exit_pos = rooms
			.choose()
			.map(|r| (r.top_left + r.bottom_right) / 2)
			.unwrap();

		// let spawn = (exit_pos * IVec2::splat(TILE_SIZE as i32)).as_vec2() +
		// Vec2::splat(TILE_SIZE as f32);

		let mut objects: Vec<_> = (0..collidable_objects.len() + background_objects.len())
			.into_iter()
			.map(|_| None)
			.collect();

		background_objects
			.into_iter()
			.chain(collidable_objects.into_iter())
			.for_each(|obj| {
				let new_obj =
					&mut objects[(obj.pos.x + obj.pos.y * MAP_WIDTH_TILES as i32) as usize];
				*new_obj = Some(obj)
			});

		let objects = objects
			.into_iter()
			.enumerate()
			.map(|(i, obj)| match obj {
				Some(obj) => obj,
				None => Object {
					pos: IVec2::new((i % MAP_WIDTH_TILES) as i32, (i / MAP_HEIGHT_TILES) as i32),
					..Default::default()
				},
			})
			.collect();

		let floor = Floor { objects };

		let mut floor_info = FloorInfo {
			monster_types: vec![MonsterObj::SmallRat(SmallRat::new(Vec2::ZERO))],
			item_types: vec![
				ItemType::Gold(20),
				ItemType::Potion(PotionType::Regeneration),
			],
			spawn,
			floor,
			rooms,
			exit: Object {
				pos: exit_pos,
				door: None,
				has_been_seen: false,
				trap: None,
				items: Vec::new(),
				is_floor: true,

				..Default::default()
			},
			monsters: Vec::new(),
		};

		floor_info.spawn_monsters();

		floor_info
	}

	pub fn rooms(&self) -> &Vec<Room> { &self.rooms }

	fn spawn_monsters(&mut self) {
		// Choose every room that doesn't contain the spawn point
		let spawn_tile = (self.spawn / Vec2::splat(TILE_SIZE as f32))
			.ceil()
			.as_ivec2();

		let valid_rooms = self.rooms.iter().filter(|room| {
			let (top_left, bottom_right) = room.extents();

			!(spawn_tile.cmpgt(top_left).all() && spawn_tile.cmplt(bottom_right).all())
		});

		self.monsters.extend(valid_rooms.flat_map(|room| {
			// Pick a random position in each room to spawn from 0 to 6 rats
			let (top_left, bottom_right) = room.extents();
			let tile_pos = IVec2::new(
				rand::gen_range(top_left.x + 1, bottom_right.x - 1),
				rand::gen_range(top_left.y + 1, bottom_right.y - 1),
			);

			let pos = (tile_pos * IVec2::splat(TILE_SIZE as i32)).as_vec2();
			let monster_types = &self.monster_types;

			(0..rand::gen_range(0, 6)).into_iter().map(move |_| {
				let monster = monster_types.choose().unwrap();
				match monster {
					MonsterObj::SmallRat(_) => MonsterObj::SmallRat(SmallRat::new(pos)),
					MonsterObj::GreenSlime(_) => MonsterObj::GreenSlime(GreenSlime::new(pos)),
				}
			})
		}));
	}

	pub fn should_descend(&self, players: &[Player]) -> bool {
		// If any players are touching the exit, descend a floor
		players
			.iter()
			.any(|p| aabb_collision(p, &self.exit, Vec2::ZERO))
	}

	pub fn exit(&self) -> &Object { &self.exit }

	pub fn current_spawn(&self) -> Vec2 { self.spawn }
}

#[derive(Clone, Serialize)]
pub struct Floor {
	objects: Vec<Object>,
}

impl Floor {
	pub fn add_item_to_object(&mut self, item: ItemInfo) {
		let object = self
			.get_object_from_pos_mut(item.tile_pos().unwrap())
			.unwrap();

		object.items.push(item);
	}

	pub fn get_object_from_pos(&self, pos: IVec2) -> Option<&Object> {
		self.objects
			.get((pos.x + pos.y * MAP_WIDTH_TILES as i32) as usize)
	}

	pub fn get_object_from_pos_mut(&mut self, pos: IVec2) -> Option<&mut Object> {
		self.objects
			.get_mut((pos.x + pos.y * MAP_WIDTH_TILES as i32) as usize)
	}

	// Same as collision, but returns the actual Object collided w.
	pub fn collision_obj<A: AsPolygon + Sync>(&self, aabb: &A, distance: Vec2) -> Option<&Object> {
		let check_collidable_obj = |object: &&Object| -> bool {
			object.is_collidable() && aabb_collision(aabb, *object, distance)
		};

		/*
		#[cfg(feature = "native")]
		return self.objects.par_iter().find_any(check_collidable_obj);

		#[cfg(not(feature = "native"))]
		*/
		self.objects.iter().find(check_collidable_obj)
	}

	pub fn collision<A: AsPolygon + Sync>(&self, aabb: &A, distance: Vec2) -> bool {
		self.collision_obj(aabb, distance).is_some()
	}

	pub fn collision_dir<A: AsPolygon + Sync>(&self, aabb: &A, distance: Vec2) -> glam::BVec2 {
		let collidable_filter = |object: &Object| -> Option<BVec2> {
			if object.is_collidable() {
				let collision_info = aabb_collision_dir(aabb, object, distance);

				match collision_info.any() {
					true => Some(collision_info),
					false => None,
				}
			} else {
				None
			}
		};

		let collision_reduction = |collision_info: BVec2, obj_collision_info: BVec2| -> BVec2 {
			collision_info | obj_collision_info
		};

		/*
		#[cfg(feature = "native")]
		return self
			.objects
			.par_iter()
			.filter_map(collidable_filter)
			.reduce(|| glam::BVec2::new(false, false), collision_reduction);

		#[cfg(not(feature = "native"))]
		*/
		const BVEC2_FALSE: BVec2 = BVec2::new(false, false);

		self.objects
			.iter()
			.filter_map(collidable_filter)
			.fold(BVEC2_FALSE, collision_reduction)
	}

	pub fn doors(&mut self) -> impl Iterator<Item = &mut Object> {
		self.objects.iter_mut().filter(|obj| obj.door.is_some())
	}

	pub fn untriggered_traps(&mut self) -> impl Iterator<Item = &mut Object> {
		self.objects.iter_mut().filter_map(|obj| match &obj.trap {
			Some(trap) => match trap.triggered {
				false => Some(obj),
				true => None,
			},
			None => None,
		})
	}

	pub fn find_path<S: AsPolygon, G: AsPolygon>(
		&self, pos: &S, goal: &G, only_visible: bool, ignore_door_collision: bool,
		randomness: Option<i32>,
	) -> Option<Vec<Vec2>> {
		inner_find_path(
			pos,
			goal,
			self,
			only_visible,
			ignore_door_collision,
			randomness,
		)
	}

	pub fn set_visible_objects<A: AsPolygon>(aabb: &A, size: Option<i32>, objects: &mut [Object]) {
		let center_tile = pos_to_tile(aabb);

		let edges = points_on_circumference(center_tile, size.unwrap_or(12));

		let rays = edges
			.into_iter()
			.map(|edge| points_on_line(center_tile, edge));

		let mut visible_object_indices =
			Vec::with_capacity(rays.len() * size.unwrap_or(12) as usize);

		for ray in rays {
			'ray: for pos in ray.into_iter() {
				if let Some(index) = get_object_from_pos_mut(pos, objects) {
					visible_object_indices.push(index);

					let obj = &objects[index];
					if obj.is_collidable() {
						break 'ray;
					}
				}
			}
		}

		visible_object_indices.iter().copied().for_each(|i| {
			objects[i].has_been_seen = true;
			objects[i].is_currently_visible = true;
		});
	}

	pub fn visible_objects<A: AsPolygon>(&self, aabb: &A, size: Option<i32>) -> Vec<&Object> {
		let center_tile = pos_to_tile(aabb);

		let edges = points_on_circumference(center_tile, size.unwrap_or(12));

		let rays = edges
			.into_iter()
			.map(|edge| points_on_line(center_tile, edge));

		let mut visible_objects = Vec::with_capacity(rays.len() * size.unwrap_or(12) as usize);

		for ray in rays {
			'ray: for pos in ray.into_iter() {
				if let Some(obj) = self.get_object_from_pos(pos) {
					visible_objects.push(obj);

					if obj.is_collidable() {
						break 'ray;
					}
				}
			}
		}

		visible_objects
	}

	pub fn objects(&self) -> &[Object] { &self.objects }

	pub fn objects_mut(&mut self) -> &mut [Object] { &mut self.objects }
}

#[derive(Clone, Serialize)]
pub struct Map {
	current_floor_index: usize,
	rooms: Vec<FloorInfo>,
}

impl Map {
	pub fn new() -> Self {
		let floors: Vec<FloorInfo> = (0..5)
			.into_iter()
			.map(|floor_num| FloorInfo::new(floor_num))
			.collect();

		Self {
			current_floor_index: 0,
			rooms: floors,
		}
	}

	pub fn current_floor(&self) -> &FloorInfo { &self.rooms[self.current_floor_index] }

	pub fn current_floor_mut(&mut self) -> &mut FloorInfo {
		&mut self.rooms[self.current_floor_index]
	}

	pub fn descend(&mut self, players: &mut [Player]) {
		self.current_floor_index += 1;
		let current_floor = self.current_floor_mut();

		players.iter_mut().for_each(|p| {
			p.pos = current_floor.spawn;
		});
	}
}

impl Drawable for Object {
	fn pos(&self) -> Vec2 { self.pos.as_vec2() * Vec2::splat(TILE_SIZE as f32) }

	fn size(&self) -> Vec2 { Vec2::splat(TILE_SIZE as f32) }

	fn texture(&self) -> Option<Texture2D> {
		Some(match self.is_floor {
			true => load_my_image("light_gray.webp"),
			false => match self.door {
				Some(door) => match door.is_open {
					false => load_my_image("door.webp"),
					true => load_my_image("open_door.webp"),
				},
				None => load_my_image("black.webp"),
			},
		})
	}
}

fn find_viable_neighbors(
	collidable_objects: &[Object], pos: IVec2, visible_objects: &Option<Vec<&Object>>,
	ignore_door_collision: bool, _randomness: Option<i32>,
) -> Vec<(IVec2, i32)> {
	let change = IVec4::new(-1, -1, 1, 1);
	let new_pos = IVec4::new(pos.x, pos.y, pos.x, pos.y) + change;

	let potential_neighbors = [
		IVec2::new(new_pos.x, pos.y),
		IVec2::new(new_pos.z, pos.y),
		IVec2::new(pos.x, new_pos.y),
		IVec2::new(pos.x, new_pos.w),
	];

	potential_neighbors
		.into_iter()
		.filter(|new_pos| {
			let p = new_pos;

			// OOB objects automatically are not eligible
			if p.cmplt(IVec2::ZERO).any() || p.cmpgt(MAP_SIZE_TILES).any() {
				false
			} else if let Some(visible_objects) = visible_objects {
				let is_visible = visible_objects.iter().any(|obj| obj.pos == *p);
				// Only return visible objects as potential neighbors
				is_visible
			} else {
				true
			}
		})
		.filter(
			|pos| match get_object_from_pos_list(*pos, collidable_objects) {
				Some(obj) => match obj.is_collidable() {
					true => ignore_door_collision && obj.door().is_some(),
					false => true,
				},
				None => true,
			},
		)
		.map(|pos| (pos, 1))
		.collect()
}

pub fn inner_find_path<S: AsPolygon, G: AsPolygon>(
	start: &S, goal: &G, floor: &Floor, only_visible: bool, ignore_door_collision: bool,
	randomness: Option<i32>,
) -> Option<Vec<Vec2>> {
	let start_tile_pos = pos_to_tile(start);
	let goal_tile_pos = pos_to_tile(goal);

	let visible_objects = match only_visible {
		true => Some(floor.visible_objects(start, None)),
		false => None,
	};

	let path = astar(
		&start_tile_pos,
		|pos| {
			find_viable_neighbors(
				&floor.objects,
				*pos,
				&visible_objects,
				ignore_door_collision,
				randomness,
			)
		},
		|pos| distance_squared(*pos, goal_tile_pos),
		|pos| *pos == goal_tile_pos,
	);

	path.map(|(positions, _)| {
		positions
			.into_iter()
			.map(|pos| (pos * IVec2::splat(TILE_SIZE as i32)).as_vec2())
			.collect()
	})
}

pub fn distance_squared(pos1: IVec2, pos2: IVec2) -> i32 {
	let mut diff = pos2 - pos1;
	diff = diff * diff;

	diff.x + diff.y
}

/// Convert from a game world position to a tile position
pub fn pos_to_tile<A: AsPolygon>(obj: &A) -> IVec2 {
	let center = obj.center();

	(center / Vec2::splat(TILE_SIZE as f32)).floor().as_ivec2()
}

pub fn trigger_traps(players: &mut [Player], floor_info: &mut FloorInfo) {
	let trapped_objs = floor_info.floor.untriggered_traps();

	trapped_objs.for_each(|trapped_obj| {
		players.iter_mut().for_each(|player| {
			let player_tile_pos = pos_to_tile(player);

			if player_tile_pos == trapped_obj.tile_pos() {
				let trap = trapped_obj.trap.as_mut().unwrap();

				trap.triggered = true;

				match trap.trap_type {
					TrapType::Teleport => {
						let rand_room = floor_info.rooms.choose().unwrap();
						let rand_pos = IVec2::new(
							rand::gen_range(rand_room.top_left.x + 1, rand_room.bottom_right.x - 1),
							rand::gen_range(rand_room.top_left.y + 1, rand_room.bottom_right.y - 1),
						);
						// Pick a random background object to teleport the player to
						player.pos = (rand_pos * IVec2::splat(TILE_SIZE as i32)).as_vec2();
					},
					TrapType::SpawnMonster => {
						// Summons six rats in the room somewhere
						floor_info.monsters.extend((0..6).into_iter().map(|_| {
							let player_room = floor_info
								.rooms
								.iter()
								.find(|room| room.inside_room(player_tile_pos))
								.unwrap();

							let tile_pos = IVec2::new(
								rand::gen_range(
									player_room.top_left.x + 1,
									player_room.bottom_right.x - 1,
								),
								rand::gen_range(
									player_room.top_left.y + 1,
									player_room.bottom_right.y - 1,
								),
							);

							let pos = (tile_pos * IVec2::splat(TILE_SIZE as i32)).as_vec2();

							match floor_info.monster_types.choose().unwrap() {
								MonsterObj::SmallRat(_) => MonsterObj::SmallRat(SmallRat::new(pos)),
								MonsterObj::GreenSlime(_) => {
									MonsterObj::GreenSlime(GreenSlime::new(pos))
								},
							}
						}))
					},
				};
			}
		});
	});
}

fn apply_effect<E: Enchantable + ?Sized>(e: &mut E, effect: EffectType) {
	let enchantment: Enchantment = effect.into();
	e.apply_enchantment(enchantment);
}

pub fn set_effects(players: &mut [Player], floor_info: &mut FloorInfo) {
	floor_info.floor.objects.iter().for_each(|obj| {
		obj.effects.keys().copied().for_each(|effect_type| {
			players
				.iter_mut()
				.for_each(|player| apply_effect(player, effect_type));

			floor_info
				.monsters
				.iter_mut()
				.for_each(|monster| apply_effect(monster, effect_type));
		});
	});
}

pub fn update_effects(floor: &mut Floor) {
	floor.objects.iter_mut().for_each(|obj| {
		obj.effects.retain(|_effect_type, effect| {
			if let Some(time_til_dissipate) = effect.time_til_dissipate.as_mut() {
				*time_til_dissipate -= 1;
				*time_til_dissipate != 0
			} else {
				true
			}
		})
	});
}

fn get_object_from_pos_mut(pos: IVec2, obj_list: &[Object]) -> Option<usize> {
	let index = (pos.x + pos.y * MAP_WIDTH_TILES as i32) as usize;

	match index < obj_list.len() {
		true => Some(index),
		false => None,
	}
}

fn get_object_from_pos_list(pos: IVec2, obj_list: &[Object]) -> Option<&Object> {
	obj_list.get((pos.x + pos.y * MAP_WIDTH_TILES as i32) as usize)
}
