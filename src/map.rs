use std::collections::HashMap;

use macroquad::prelude::*;
use macroquad::rand::*;
use pathfinding::prelude::*;

use crate::draw::Drawable;
use crate::math::points_on_circumference;
use crate::math::points_on_line;
use crate::math::{aabb_collision, AsAABB, AxisAlignedBoundingBox};
use crate::monsters::{Monster, SmallRat};
use crate::player::Player;
use crate::player::PLAYER_SIZE;
use crate::PLAYER_AABB;

pub const TILE_SIZE: usize = 25;

pub const MAP_WIDTH_TILES: usize = 80;
pub const MAP_HEIGHT_TILES: usize = 80;

pub const MAP_SIZE_TILES: IVec2 = IVec2::new(MAP_WIDTH_TILES as i32, MAP_HEIGHT_TILES as i32);

#[derive(Copy, Clone)]
pub struct Object {
	pos: IVec2,
	texture: Texture2D,
	is_floor: bool,
	door: Option<Door>,
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

	/// Returns whether or not a position is inside a room
	fn inside_room(&self, pos: IVec2) -> bool {
		pos.cmpgt(self.top_left).all() && pos.cmplt(self.bottom_right).all()
	}
}

pub struct Floor {
	spawn: Vec2,
	rooms: Vec<Room>,
	pub collidable_objects: Vec<Object>,
	background_objects: Vec<Object>,
	exit: Object,
}

impl Floor {
	pub fn background_objects(&self) -> &[Object] {
		&self.background_objects
	}

	// Same as collision, but returns the actual Object collided w.
	pub fn collision_obj<A: AsAABB>(&self, aabb: &A, distance: Vec2) -> Option<&Object> {
		self.collidable_objects
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
						is_floor: false,
					},
					Object {
						pos: IVec2::new(x, MAP_HEIGHT_TILES as i32),
						texture: *textures.get("black.webp").unwrap(),
						door: None,
						is_floor: false,
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
						is_floor: false,
					},
					Object {
						pos: IVec2::new(MAP_WIDTH_TILES as i32, y),
						texture: *textures.get("black.webp").unwrap(),
						door: None,
						is_floor: false,
					},
				]
				.into_iter()
			}))
		});

		let room_walls = rooms
			.iter()
			.flat_map(|room: &Room| room.generate_walls())
			.map(|w_pos| {
				let door = rooms
					.iter()
					.find_map(|r| r.doors.iter().find(|d| d.pos == w_pos))
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
					door,
				}
			});

		let rooms_ref = &rooms;
		let hallways_ref = &hallways;

		let (dungeon_walls, floor): (Vec<_>, Vec<_>) =
			((0..MAP_WIDTH_TILES).into_iter().flat_map(|x| {
				(0..MAP_HEIGHT_TILES).into_iter().map(move |y| {
					let pos = IVec2::new(x as i32, y as i32);
					let in_room = rooms_ref.iter().any(|r| r.inside_room(pos));
					let is_hallway = hallways_ref.iter().any(|h| *h == pos);

					let is_dungeon_wall = !in_room && !is_hallway;
					(pos, is_dungeon_wall)
				})
			}))
			.partition(|(_pos, is_dungeon_wall)| *is_dungeon_wall);

		let pos_to_obj = |&(pos, is_dungeon_wall): &(IVec2, bool)| -> Object {
			Object {
				pos,
				texture: *textures
					.get(match is_dungeon_wall {
						true => "black.webp",
						false => "light_gray.webp",
					})
					.unwrap(),
				door: None,
				is_floor: !is_dungeon_wall,
			}
		};

		let collidable_objects = walls
			.chain(room_walls)
			.chain(dungeon_walls.iter().map(pos_to_obj))
			.collect();

		let background_objects = hallways
			.iter()
			.map(|&pos| Object {
				pos,
				texture: *textures.get("light_gray.webp").unwrap(),
				door: None,
				is_floor: true,
			})
			.chain(floor.iter().map(pos_to_obj))
			.collect();

		let spawn = rooms
			.choose()
			.map(|r| {
				(((r.top_left + r.bottom_right) / 2) * IVec2::splat(TILE_SIZE as i32)).as_vec2()
			})
			.unwrap();

		Floor {
			spawn,
			collidable_objects,
			background_objects,
			rooms,
			exit: Object {
				pos: IVec2::ZERO,
				texture: *textures.get("green.webp").unwrap(),
				door: None,
				is_floor: true,
			},
		}
	}

	pub fn doors(&mut self) -> impl Iterator<Item = &mut Door> {
		self.collidable_objects
			.iter_mut()
			.filter_map(|obj| obj.door.as_mut())
	}

	pub fn find_path(&self, pos: &dyn AsAABB, goal: &dyn AsAABB) -> Option<Vec<Vec2>> {
		find_path(pos, goal, &self.collidable_objects)
	}

	pub fn should_descend(&self, players: &[Player], _monsters: &[Box<dyn Monster>]) -> bool {
		// If any players are touching the exit, descend a floor
		players
			.iter()
			.any(|p| aabb_collision(p, &self.exit, Vec2::ZERO))
	}

	pub fn visible_objects(&self, aabb: &dyn AsAABB) -> Vec<&Object> {
		let center_tile = pos_to_tile(aabb);

		let edges = points_on_circumference(center_tile, 16);

		let mut visible_objects: Vec<&Object> = Vec::new();

		let rays = edges
			.into_iter()
			.map(|edge| points_on_line(center_tile, edge));

		let objects = || {
			self.collidable_objects
				.iter()
				.chain(self.background_objects.iter())
		};

		for ray in rays {
			'ray: for pos in ray.into_iter() {
				if let Some(obj) = objects().find(|obj| obj.pos == pos) {
					visible_objects.push(obj);

					if obj.is_collidable() {
						break 'ray;
					}
				}
			}
		}

		visible_objects

		// let center_tile = pos_to_tile(aabb);
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

	fn draw(&self) {
		let floor = self.current_floor();

		floor
			.visible_objects(unsafe { &PLAYER_AABB })
			.into_iter()
			.filter(|o| match &o.door {
				Some(door) => !door.is_open,
				None => true,
			})
			.for_each(|o| o.draw());
		floor.exit.draw();
	}
}

fn find_viable_neighbors(collidable_objects: &[Object], pos: IVec2) -> Vec<(IVec2, i32)> {
	let change = IVec4::new(-1, -1, 1, 1);
	let new_pos = IVec4::new(pos.x, pos.y, pos.x, pos.y) + change;

	let mut potential_neighbors = [
		Some(IVec2::new(new_pos.x, pos.y)),
		Some(IVec2::new(new_pos.z, pos.y)),
		Some(IVec2::new(pos.x, new_pos.y)),
		Some(IVec2::new(pos.x, new_pos.w)),
	];

	potential_neighbors.iter_mut().for_each(|new_pos| {
		let p = unsafe { new_pos.unwrap_unchecked() };

		if p.cmplt(IVec2::ZERO).any() || p.cmpgt(MAP_SIZE_TILES).any() {
			*new_pos = None;
		}
	});

	collidable_objects.iter().for_each(|c| {
		potential_neighbors.iter_mut().for_each(|p| {
			if let Some(p_clone) = *p {
				// If any potential_neighbors are a collidable object, remove them from the pool
				if c.is_collidable() && p_clone == c.pos {
					*p = None;
				}
			}
		})
	});

	potential_neighbors
		.iter()
		.filter_map(|p| p.map(|p| (p, 1)))
		.collect()
}

pub fn find_path(
	start: &dyn AsAABB, goal: &dyn AsAABB, collidable_objects: &[Object],
) -> Option<Vec<Vec2>> {
	let aabb = start.as_aabb();

	let start_tile_pos = pos_to_tile(start);
	let goal_tile_pos = pos_to_tile(goal);

	let path = astar(
		&start_tile_pos,
		|pos| find_viable_neighbors(collidable_objects, *pos),
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
	monsters.extend((0..35).map(|_| {
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
