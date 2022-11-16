use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::slice;

use macroquad::prelude::*;
use macroquad::rand;
use macroquad::rand::*;
use pathfinding::prelude::*;
use rayon::prelude::*;

use crate::draw::Drawable;
use crate::math::points_on_circumference;
use crate::math::points_on_line;
use crate::math::{aabb_collision, AsAABB, AxisAlignedBoundingBox};
use crate::monsters::{Monster, SmallRat};
use crate::player::Player;

pub const TILE_SIZE: usize = 25;

pub const MAP_WIDTH_TILES: usize = 80;
pub const MAP_HEIGHT_TILES: usize = 80;

pub const MAP_SIZE_TILES: IVec2 = IVec2::new(MAP_WIDTH_TILES as i32, MAP_HEIGHT_TILES as i32);

#[derive(Copy, Clone)]
enum TrapType {
	Teleport,
}

#[derive(Copy, Clone)]
struct Trap {
	triggered: bool,
	trap_type: TrapType,
}

#[derive(Copy, Clone)]
pub struct Object {
	pos: IVec2,
	texture: Texture2D,
	is_floor: bool,
	has_been_seen: bool,
	door: Option<Door>,
	trap: Option<Trap>,
}

impl PartialEq for Object {
	fn eq(&self, other: &Self) -> bool {
		self.pos == other.pos
	}
}

impl Object {
	pub fn tile_pos(&self) -> IVec2 {
		self.pos
	}

	fn is_collidable(&self) -> bool {
		if self.is_floor {
			return false;
		}

		match &self.door {
			Some(door) => !door.is_open,
			None => true,
		}
	}

	pub fn door(&self) -> &Option<Door> {
		&self.door
	}

	pub fn has_been_seen(&self) -> bool {
		self.has_been_seen
	}

	pub fn open_door(&mut self, textures: &HashMap<String, Texture2D>) {
		if let Some(door) = &mut self.door {
			self.texture = *textures.get("open_door.webp").unwrap();
			door.open();
		}
	}

	pub fn close_door(&mut self, textures: &HashMap<String, Texture2D>) {
		if let Some(door) = &mut self.door {
			self.texture = *textures.get("door.webp").unwrap();
			door.close();
		}
	}
}

impl AsAABB for Object {
	fn as_aabb(&self) -> AxisAlignedBoundingBox {
		AxisAlignedBoundingBox {
			pos: self.pos.as_vec2() * Vec2::splat(TILE_SIZE as f32),
			size: Vec2::splat(TILE_SIZE as f32),
		}
	}
}

#[derive(Copy, Clone)]
pub struct Door {
	pos: IVec2,
	pub is_open: bool,
}

impl Door {
	pub fn pos(&self) -> IVec2 {
		self.pos
	}

	pub fn is_open(&self) -> bool {
		self.is_open
	}

	pub fn open(&mut self) {
		self.is_open = true;
	}

	pub fn close(&mut self) {
		self.is_open = false;
	}
}

struct Room {
	top_left: IVec2,
	bottom_right: IVec2,
	doors: Vec<Door>,
}

impl Room {
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

	fn generate_wall_objects(&self, textures: &HashMap<String, Texture2D>) -> Vec<Object> {
		self.generate_walls()
			.into_iter()
			.map(|w_pos| {
				let door = self
					.doors
					.iter()
					.find(|d| d.pos == w_pos)
					.map(|d| d.clone());

				let texture = *textures
					.get(match door.is_some() {
						true => "door.webp",
						false => "black.webp",
					})
					.unwrap();

				Object {
					pos: w_pos,
					texture,
					is_floor: false,
					trap: None,
					has_been_seen: false,
					door,
				}
			})
			.collect()
	}

	fn generate_floor(&self, textures: &HashMap<String, Texture2D>) -> Vec<Object> {
		let map_object = |x: i32, y: i32| -> Object {
			// 1 in 250 chance of being a trapped tile
			let is_trap: bool = rand::gen_range(1, 250) == 100;

			let trap = match is_trap {
				true => Some(Trap {
					triggered: false,
					trap_type: TrapType::Teleport,
				}),
				false => None,
			};

			let texture = match is_trap {
				true => *textures.get("trap.webp").unwrap(),
				false => *textures.get("light_gray.webp").unwrap(),
			};

			Object {
				pos: IVec2::new(x, y),
				texture,
				is_floor: true,
				has_been_seen: false,
				door: None,
				trap,
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
}

pub struct Floor {
	spawn: Vec2,
	rooms: Vec<Room>,
	pub objects: Vec<Object>,
	exit: Object,
}

impl Floor {
	pub fn get_object_from_pos(&self, pos: IVec2) -> Option<&Object> {
		self.objects
			.get((pos.x + pos.y * MAP_WIDTH_TILES as i32) as usize)
	}

	pub fn get_object_from_pos_mut(&mut self, pos: IVec2) -> Option<&mut Object> {
		self.objects
			.get_mut((pos.x + pos.y * MAP_WIDTH_TILES as i32) as usize)
	}

	pub fn background_objects(&self) -> impl Iterator<Item = &Object> {
		self.objects.iter().filter(|obj| !obj.is_collidable())
	}

	// Same as collision, but returns the actual Object collided w.
	pub fn collision_obj<A: AsAABB>(&self, aabb: &A, distance: Vec2) -> Option<&Object> {
		self.objects
			.iter()
			.find(|object| object.is_collidable() && aabb_collision(aabb, *object, distance))
	}

	pub fn collision<A: AsAABB>(&self, aabb: &A, distance: Vec2) -> bool {
		self.collision_obj(aabb, distance).is_some()
	}

	pub fn current_spawn(&self) -> Vec2 {
		self.spawn
	}

	pub fn new(_floor_num: usize, textures: &HashMap<String, Texture2D>) -> Self {
		let mut rooms = Vec::new();

		// First, try to flll the map with as many rooms as possible
		for _ in 0..100_000 {
			const MIN_SIZE: i32 = 8;
			const MAX_SIZE: i32 = 14;

			let top_left = IVec2::new(
				rand::gen_range(0, MAP_WIDTH_TILES as i32),
				rand::gen_range(0, MAP_HEIGHT_TILES as i32),
			);
			let bottom_right = top_left
				+ IVec2::new(
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
				const MIN_DISTANCE_BETWEEN_ROOMS: i32 = 1;

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

		let mut hallways: Vec<IVec2> = rooms[0..(rooms.len() * 3) / 4]
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
					if door_pos != room.top_left
						&& door_pos != room.bottom_right
						&& door_pos != IVec2::new(room.bottom_right.x, room.top_left.y)
						&& door_pos != IVec2::new(room.top_left.x, room.bottom_right.y)
					{
						room.doors.push(Door {
							pos: door_pos,
							is_open: false,
						});
					}
				});
		});

		// Only keep rooms that have a door
		rooms.retain(|room| room.doors.len() > 0);

		// Remove all "hallway" positions inside of a room
		hallways.retain(|h| !rooms.iter().any(|r| r.inside_room(*h)));

		// Actually render all of the walls
		let walls = (0..MAP_WIDTH_TILES as i32).into_iter().flat_map(|x| {
			{
				[
					Object {
						pos: IVec2::new(x, 0),
						texture: *textures.get("black.webp").unwrap(),
						door: None,
						has_been_seen: false,
						is_floor: false,
						trap: None,
					},
					Object {
						pos: IVec2::new(x, MAP_HEIGHT_TILES as i32),
						texture: *textures.get("black.webp").unwrap(),
						door: None,
						has_been_seen: false,
						is_floor: false,
						trap: None,
					},
				]
				.into_iter()
			}
			.chain((0..=MAP_HEIGHT_TILES as i32).into_iter().flat_map(|y| {
				[
					Object {
						pos: IVec2::new(0, y),
						texture: *textures.get("black.webp").unwrap(),
						door: None,
						has_been_seen: false,
						is_floor: false,
						trap: None,
					},
					Object {
						pos: IVec2::new(MAP_WIDTH_TILES as i32, y),
						texture: *textures.get("black.webp").unwrap(),
						door: None,
						has_been_seen: false,
						is_floor: false,
						trap: None,
					},
				]
				.into_iter()
			}))
		});

		let room_walls = rooms
			.iter()
			.flat_map(|room: &Room| room.generate_wall_objects(&textures));

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
						texture: *textures.get("black.webp").unwrap(),
						is_floor: false,
						has_been_seen: false,
						door: None,
						trap: None,
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
				texture: *textures.get("light_gray.webp").unwrap(),
				door: None,
				has_been_seen: false,
				trap: None,
				is_floor: true,
			})
			.chain(rooms.iter().flat_map(|r| r.generate_floor(textures)))
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

		// let spawn = (exit_pos * IVec2::splat(TILE_SIZE as i32)).as_vec2() + Vec2::splat(TILE_SIZE as f32);

		let mut objects =
			vec![MaybeUninit::uninit(); collidable_objects.len() + background_objects.len()];

		background_objects
			.into_iter()
			.chain(collidable_objects.into_iter())
			.for_each(|obj| {
				let new_obj =
					&mut objects[(obj.pos.x + obj.pos.y * MAP_WIDTH_TILES as i32) as usize];
				new_obj.write(obj);
			});

		let objects = objects
			.into_iter()
			.map(|obj| unsafe { obj.assume_init() })
			.collect();

		Floor {
			spawn,
			objects,
			rooms,
			exit: Object {
				pos: exit_pos,
				texture: *textures.get("green.webp").unwrap(),
				door: None,
				has_been_seen: false,
				trap: None,
				is_floor: true,
			},
		}
	}

	pub fn exit(&self) -> &Object {
		&self.exit
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

	pub fn find_path(
		&self, pos: &dyn AsAABB, goal: &dyn AsAABB, only_visible: bool,
	) -> Option<Vec<Vec2>> {
		find_path(pos, goal, &self, only_visible)
	}

	pub fn should_descend(&self, players: &[Player], _monsters: &[Box<dyn Monster>]) -> bool {
		// If any players are touching the exit, descend a floor
		players
			.iter()
			.any(|p| aabb_collision(p, &self.exit, Vec2::ZERO))
	}

	pub fn visible_objects_mut<'a>(
		aabb: &dyn AsAABB, size: Option<i32>, objects: &'a mut [Object],
	) -> Vec<&'a Object> {
		let center_tile = pos_to_tile(aabb);

		let edges = points_on_circumference(center_tile, size.unwrap_or(12));

		let rays = edges
			.into_iter()
			.map(|edge| points_on_line(center_tile, edge));

		let mut visible_objects = Vec::with_capacity(rays.len() * size.unwrap_or(12) as usize);

		for ray in rays {
			'ray: for pos in ray.into_iter() {
				if let Some(obj) = get_object_from_pos_mut(pos, unsafe {
					// Evil borrow laundering
					slice::from_raw_parts_mut(objects.as_mut_ptr(), objects.len())
				}) {
					obj.has_been_seen = true;
					let obj = &*obj;
					visible_objects.push(obj);

					if obj.is_collidable() {
						break 'ray;
					}
				}
			}
		}

		visible_objects
	}

	pub fn visible_objects(&self, aabb: &dyn AsAABB, size: Option<i32>) -> Vec<&Object> {
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
}

pub struct Map {
	current_floor_index: usize,
	rooms: Vec<Floor>,
}

impl Map {
	pub fn new(
		textures: &HashMap<String, Texture2D>, monsters: &mut Vec<Box<dyn Monster>>,
	) -> Self {
		let floors: Vec<Floor> = (0..5)
			.into_iter()
			.map(|floor_num| Floor::new(floor_num, textures))
			.collect();

		let map = Self {
			current_floor_index: 0,
			rooms: floors,
		};

		spawn_monsters(0, monsters, textures, map.current_floor());

		map
	}

	pub fn current_floor(&self) -> &Floor {
		&self.rooms[self.current_floor_index]
	}

	pub fn current_floor_mut(&mut self) -> &mut Floor {
		&mut self.rooms[self.current_floor_index]
	}

	pub fn descend(
		&mut self, players: &mut [Player], monsters: &mut Vec<Box<dyn Monster>>,
		textures: &HashMap<String, Texture2D>,
	) {
		self.current_floor_index += 1;
		let current_floor = self.current_floor();

		players.iter_mut().for_each(|p| {
			p.pos = current_floor.spawn;
		});

		monsters.clear();

		spawn_monsters(self.current_floor_index, monsters, textures, current_floor);
	}
}

impl Drawable for Object {
	fn pos(&self) -> Vec2 {
		self.pos.as_vec2() * Vec2::splat(TILE_SIZE as f32)
	}

	fn size(&self) -> Vec2 {
		Vec2::splat(TILE_SIZE as f32)
	}

	fn texture(&self) -> Option<Texture2D> {
		Some(self.texture)
	}
}

impl Drawable for Map {
	fn pos(&self) -> Vec2 {
		Vec2::ZERO
	}

	fn size(&self) -> Vec2 {
		Vec2::ZERO
	}

	fn draw(&self) {}
}

fn find_viable_neighbors(
	collidable_objects: &[Object], pos: IVec2, visible_objects: &Option<Vec<&Object>>,
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
				Some(obj) => !obj.is_collidable(),
				None => true,
			},
		)
		.map(|pos| (pos, 1))
		.collect()
}

pub fn find_path(
	start: &dyn AsAABB, goal: &dyn AsAABB, floor: &Floor, only_visible: bool,
) -> Option<Vec<Vec2>> {
	let aabb = start.as_aabb();

	let start_tile_pos = pos_to_tile(start);
	let goal_tile_pos = pos_to_tile(goal);

	let visible_objects = match only_visible {
		true => Some(floor.visible_objects(start, None)),
		false => None,
	};

	let path = astar(
		&start_tile_pos,
		|pos| find_viable_neighbors(&floor.objects, *pos, &visible_objects),
		|pos| distance_squared(*pos, goal_tile_pos),
		|pos| *pos == goal_tile_pos,
	);

	path.map(|(positions, _)| {
		positions
			.iter()
			.map(|pos| (*pos * IVec2::splat(TILE_SIZE as i32)).as_vec2() + (aabb.size * 0.5))
			.collect()
	})
}

fn spawn_monsters(
	_floor_num: usize, monsters: &mut Vec<Box<dyn Monster>>, textures: &HashMap<String, Texture2D>,
	floor: &Floor,
) {
	monsters.extend((0..85).map(|_| {
		let monster: Box<dyn Monster> = Box::new(SmallRat::new(textures, floor));
		monster
	}));
}

pub fn distance_squared(pos1: IVec2, pos2: IVec2) -> i32 {
	let mut diff = pos2 - pos1;
	diff = diff * diff;

	diff.x + diff.y
}

/// Convert from a game world position to a tile position
pub fn pos_to_tile(obj: &dyn AsAABB) -> IVec2 {
	let center = obj.center();

	let tile_pos = (center / Vec2::splat(TILE_SIZE as f32)).round().as_ivec2();
	tile_pos
}

pub fn trigger_traps(players: &mut [Player], floor: &mut Floor) {
	let rand_room = floor.rooms.choose().unwrap();
	let rand_pos = IVec2::new(
		rand::gen_range(rand_room.top_left.x + 1, rand_room.bottom_right.x - 1),
		rand::gen_range(rand_room.top_left.y + 1, rand_room.bottom_right.y - 1),
	);

	let trapped_objs = floor.untriggered_traps();

	trapped_objs.for_each(|trapped_obj| {
		players.iter_mut().for_each(|player| {
			let player_tile_pos = pos_to_tile(player);

			if player_tile_pos == trapped_obj.tile_pos() {
				let trap = &mut trapped_obj.trap.unwrap();

				trap.triggered = true;

				match trap.trap_type {
					TrapType::Teleport => {
						// Pick a random background object to teleport the player to
						player.pos = (rand_pos * IVec2::splat(TILE_SIZE as i32)).as_vec2();
					},
				};
			}
		});
	});
}

fn get_object_from_pos_mut(pos: IVec2, obj_list: &mut [Object]) -> Option<&mut Object> {
	obj_list.get_mut((pos.x + pos.y * MAP_WIDTH_TILES as i32) as usize)
}

fn get_object_from_pos_list(pos: IVec2, obj_list: &[Object]) -> Option<&Object> {
	obj_list.get((pos.x + pos.y * MAP_WIDTH_TILES as i32) as usize)
}
