use std::collections::HashMap;

use macroquad::prelude::*;
use macroquad::rand::*;
use pathfinding::prelude::*;

use crate::draw::Drawable;
use crate::math::{aabb_collision, AsAABB, AxisAlignedBoundingBox};
use crate::monsters::{Monster, SmallRat};
use crate::player::Player;

pub const TILE_SIZE: usize = 25;
pub const MAP_WIDTH_TILES: usize = 80;
pub const MAP_HEIGHT_TILES: usize = 80;

pub struct Object {
	pos: IVec2,
	texture: Texture2D,
	is_door: bool,
}

impl AsAABB for Object {
	fn as_aabb(&self) -> AxisAlignedBoundingBox {
		AxisAlignedBoundingBox {
			pos: self.pos.as_vec2() * Vec2::splat(TILE_SIZE as f32),
			size: Vec2::splat(TILE_SIZE as f32),
		}
	}
}

struct Room {
	top_left: IVec2,
	bottom_right: IVec2,
	doors: Vec<IVec2>,
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
	collidable_objects: Vec<Object>,
	background_objects: Vec<Object>,
	exit: Object,
}

impl Floor {
	pub fn background_objects(&self) -> &[Object] { &self.background_objects }

	// Same as collision, but returns the actual Object collided w.
	pub fn collision_obj<A: AsAABB>(&self, aabb: &A, distance: Vec2) -> Option<&Object> {
		self.collidable_objects
			.iter()
			.find(|object| aabb_collision(aabb, *object, distance))
	}

	pub fn collision<A: AsAABB>(&self, aabb: &A, distance: Vec2) -> bool {
		self.collision_obj(aabb, distance).is_some()
	}

	pub fn current_spawn(&self) -> Vec2 { self.spawn }

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

				let center_touching_wall = |room: &Room, center: IVec2| {
					center.cmpeq(room.top_left).any() || center.cmpeq(room.bottom_right).any()
				};

				if rooms
					.iter()
					.any(|room| center_touching_wall(room, left_room))
				{
					let new_left_room = (-2..=2).into_iter().find_map(|change| {
						let left_room_x_change = left_room + IVec2::new(change, 0);
						let left_room_y_change = left_room + IVec2::new(0, change);

						rooms.iter().find_map(|room_extremes| {
							if !center_touching_wall(room_extremes, left_room_x_change) {
								Some(left_room_x_change)
							} else if !center_touching_wall(room_extremes, left_room_y_change) {
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
					.any(|room_extremes| center_touching_wall(room_extremes, right_room))
				{
					let new_right_room = (-2..=2).into_iter().find_map(|change| {
						let right_room_x_change = right_room + IVec2::new(change, 0);
						let right_room_y_change = right_room + IVec2::new(0, change);

						rooms.iter().find_map(|room_extremes| {
							if !center_touching_wall(room_extremes, right_room_x_change) {
								Some(right_room_x_change)
							} else if !center_touching_wall(room_extremes, right_room_y_change) {
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
						room.doors.push(door_pos);
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
						is_door: false,
					},
					Object {
						pos: IVec2::new(x, MAP_HEIGHT_TILES as i32),
						texture: *textures.get("black.webp").unwrap(),
						is_door: false,
					},
				]
				.into_iter()
			}
			.chain((0..=MAP_HEIGHT_TILES as i32).into_iter().flat_map(|y| {
				[
					Object {
						pos: IVec2::new(0, y),
						texture: *textures.get("black.webp").unwrap(),
						is_door: false,
					},
					Object {
						pos: IVec2::new(MAP_WIDTH_TILES as i32, y),
						texture: *textures.get("black.webp").unwrap(),
						is_door: false,
					},
				]
				.into_iter()
			}))
		});

		let room_walls = rooms
			.iter()
			.flat_map(|room: &Room| room.generate_walls())
			.map(|w_pos| {
				let is_door = rooms.iter().any(|r| r.doors.contains(&w_pos));
				let texture = *textures
					.get(match is_door {
						true => "door.webp",
						false => "black.webp",
					})
					.unwrap();

				Object {
					pos: w_pos,
					texture,
					is_door,
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
				is_door: false,
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
				is_door: false,
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
			exit: Object {
				pos: IVec2::ZERO,
				texture: *textures.get("green.webp").unwrap(),
				is_door: false,
			},
		}
	}

	pub fn find_path(&self, pos: &dyn AsAABB, goal: Vec2) -> Option<Vec<Vec2>> {
		find_path(pos, goal, &self.collidable_objects)
	}

	pub fn should_descend(&self, players: &[Player], _monsters: &[Box<dyn Monster>]) -> bool {
		// If any players are touching the exit, descend a floor
		players
			.iter()
			.any(|p| aabb_collision(p, &self.exit, Vec2::ZERO))
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

	pub fn current_floor(&self) -> &Floor { &self.rooms[self.current_floor_index] }

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
	fn pos(&self) -> Vec2 { self.pos.as_vec2() * Vec2::splat(TILE_SIZE as f32) }

	fn size(&self) -> Vec2 { Vec2::splat(TILE_SIZE as f32) }

	fn texture(&self) -> Option<Texture2D> { Some(self.texture) }
}

impl Drawable for Map {
	fn pos(&self) -> Vec2 { Vec2::ZERO }

	fn size(&self) -> Vec2 { Vec2::ZERO }

	fn draw(&self) {
		let room = &self.rooms[self.current_floor_index];
		room.background_objects
			.iter()
			.chain(room.collidable_objects.iter())
			.for_each(|o| o.draw());
		room.exit.draw();
	}
}

fn find_viable_neighbors(collidable_objects: &[Object], pos: IVec2) -> Vec<(IVec2, i32)> {
	let mut potential_neighbors = {
		let mut positions = [Some(pos); 4];
		let changes = [
			IVec2::new(-1, 0),
			IVec2::new(1, 0),
			IVec2::new(0, -1),
			IVec2::new(0, 1),
		];

		positions
			.iter_mut()
			.zip(changes.into_iter())
			.for_each(|(p, change)| {
				let new_pos = p.unwrap() + change;

				if new_pos.x < 0 ||
					new_pos.x >= MAP_WIDTH_TILES.try_into().unwrap() ||
					new_pos.y < 0 || new_pos.y >= MAP_HEIGHT_TILES.try_into().unwrap()
				{
					*p = None;
				} else {
					*p = Some(new_pos);
				}
			});

		positions
	};

	collidable_objects.iter().for_each(|c| {
		potential_neighbors.iter_mut().for_each(|p| {
			if let Some(p_clone) = *p {
				// If any potential_neighbors are a collidable object, remove them from the pool
				if p_clone == c.pos {
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
	start: &dyn AsAABB, goal: Vec2, collidable_objects: &[Object],
) -> Option<Vec<Vec2>> {
	let aabb = start.as_aabb();

	let start_tile_pos = (aabb.pos / Vec2::splat(TILE_SIZE as f32)).as_ivec2();
	let goal_tile_pos = (goal / Vec2::splat(TILE_SIZE as f32)).as_ivec2();

	let path = astar(
		&start_tile_pos,
		|pos| find_viable_neighbors(collidable_objects, *pos),
		|pos| distance_squared(*pos, goal_tile_pos),
		|pos| *pos == goal_tile_pos,
	);

	path.map(|(positions, _)| {
		positions
			.iter()
			.map(|pos| (*pos * IVec2::splat(TILE_SIZE as i32)).as_vec2() - (aabb.size * 0.25))
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

fn distance(pos1: IVec2, pos2: IVec2) -> f32 {
	let distance_squared = distance_squared(pos1, pos2);

	(distance_squared as f32).sqrt()
}

fn distance_squared(pos1: IVec2, pos2: IVec2) -> i32 {
	let mut diff = pos2 - pos1;
	diff = diff * diff;

	diff.x + diff.y
}
