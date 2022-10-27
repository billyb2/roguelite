use std::collections::HashMap;

use macroquad::prelude::*;
use here_be_dragons::*;
use ordered_float::OrderedFloat;
use pathfinding::prelude::*;

use crate::{
    draw::Drawable, 
    math::{AsAABB, aabb_collision, AxisAlignedBoundingBox}, monsters::{Monster, SmallRat}};

pub struct Object {
    index: usize,
    pos: Vec2,
    size: Vec2,
    texture: Texture2D,
    is_exit: bool,

}

impl AsAABB for Object {
    fn as_aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox { 
            pos: self.pos, 
            size: self.size,
        }
        
    }

}

pub struct Room {
    spawn: Vec2,
    collidable_objects: Vec<Object>,
    background_objects: Vec<Object>,

}

impl Room {
    pub fn background_objects(&self) -> &[Object] {
        &self.background_objects

    }

}

pub struct Map {
    current_room_index: usize,
    rooms: Vec<Room>,

}

pub const TILE_SIZE: usize = 20;
const TILE_WIDTH: usize = 30;
const TILE_HEIGHT: usize = 30;

impl Map {
    pub fn new(textures: &HashMap<String, Texture2D>, monsters: &mut Vec<Box<dyn Monster>>) -> Self {
        let map_gen = MapBuilder::<NoData>::new(TILE_WIDTH, TILE_HEIGHT)
            .with(NoiseGenerator::new(0.1))
            .with(BspRooms::new())
            .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
            .with(DistantExit::new())
            .build();

        let tile_to_object = |i: usize, texture: Texture2D| -> Object {
            let x_pos = (i % TILE_WIDTH) * TILE_SIZE;
            let y_pos = (i / TILE_HEIGHT) * TILE_SIZE;
            
            Object {
                index: i,
                pos: Vec2::new(x_pos as f32, y_pos as f32),
                size: Vec2::splat(TILE_SIZE as f32),
                texture,
                is_exit: false,

            }

        };

        let (collidable_objects, background_objects): (Vec<_>, Vec<_>) = map_gen.tiles.iter().enumerate().map(|(i, t)| (t.is_blocked(), i)).partition(|(is_blocked, _)| *is_blocked);

        let collidable_object_texture = *textures.get("black.webp").unwrap();
        let background_object_texture = *textures.get("light_gray.webp").unwrap();

        let collidable_objects: Vec<Object> = collidable_objects.into_iter().map(|(_, i)| tile_to_object(i, collidable_object_texture)).collect();
        let mut background_objects: Vec<Object> = background_objects.into_iter().map(|(_, i)| tile_to_object(i, background_object_texture)).collect();

        background_objects.push(Object {
            index: background_objects.len(),
            pos: map_gen.exit_point.map(|p| Vec2::new(p.x as f32, p.y as f32) * Vec2::splat(TILE_SIZE as f32)).unwrap(),
            size: Vec2::splat(TILE_SIZE as f32),
            texture: *textures.get("green.webp").unwrap(),
            is_exit: true,

        });

        let rooms = vec![Room {
            spawn: map_gen.starting_point.map(|p| Vec2::new(p.x as f32, p.y as f32) * Vec2::splat(TILE_SIZE as f32)).unwrap(),
            collidable_objects,
            background_objects,

        }];

        let map = Self {
            current_room_index: 0,
            rooms,

        };

        monsters.extend((0..10).map(|_| {
            let monster: Box<dyn Monster> = Box::new(SmallRat::new(textures, &map));
            monster
        }));

        map

    }

    pub fn current_spawn(&self) -> Vec2 {
        self.rooms[self.current_room_index].spawn

    }

    pub fn collision(&self, aabb: &dyn AsAABB, distance: Vec2) -> bool {
        let current_room = &self.rooms[self.current_room_index];

        current_room.collidable_objects.iter().any(|object| 
            aabb_collision(aabb, object, distance)

        )

    }

    pub fn find_viable_neighbors(&self, tile_index: usize) -> Vec<(usize, OrderedFloat<f32>)> {
        let current_room = &self.rooms[self.current_room_index];

        [-1, 1, (TILE_WIDTH as isize), -(TILE_WIDTH as isize)].into_iter().filter_map(|change| {
            let tile_index = tile_index as isize + change;

            if tile_index < 0 || tile_index > (TILE_WIDTH * TILE_HEIGHT) as isize {
                return None;

            }

            if let Some(o) = current_room.background_objects.iter().find(|o| o.index == tile_index as usize) {
                Some((tile_index as usize, self.tile_distance(tile_index as usize, o.index)))

            } else {
                None

            }

        }).collect()

    }

    pub fn current_room(&self) -> &Room {
        &self.rooms[self.current_room_index]

    }

    pub fn tile_distance(&self, tile_index: usize, other_tile_index: usize) -> OrderedFloat<f32> {
        let x_pos = (tile_index % TILE_WIDTH) * TILE_SIZE;
        let y_pos = (tile_index / TILE_HEIGHT) * TILE_SIZE;

        let tile1 = Vec2::new(x_pos as f32, y_pos as f32);

        let x_pos = (other_tile_index % TILE_WIDTH) * TILE_SIZE;
        let y_pos = (other_tile_index / TILE_HEIGHT) * TILE_SIZE;

        let tile2 = Vec2::new(x_pos as f32, y_pos as f32);

        OrderedFloat(tile1.distance_squared(tile2))

    }

    pub fn find_path(&self, start: Vec2, goal: Vec2) -> Option<Vec<Vec2>> {
        // First, try to find the tile the starting position is in
        let start_tile = (start / Vec2::splat(TILE_SIZE as f32)).floor();
        let goal_tile = (goal / Vec2::splat(TILE_SIZE as f32)).floor();

        let start_tile_index = start_tile.x as usize + start_tile.y as usize * TILE_WIDTH;
        let goal_tile_index = goal_tile.x as usize + goal_tile.y as usize * TILE_WIDTH;

        let path = astar(
            &start_tile_index, 
            |i| self.find_viable_neighbors(*i), 
            |i| self.tile_distance(*i, goal_tile_index), 
            |i| *i == goal_tile_index
        );

        path.map(|(indices, _)| indices.iter().map(|i| {
            let x_pos = (i % TILE_WIDTH) * TILE_SIZE;
            let y_pos = (i / TILE_HEIGHT) * TILE_SIZE;

            Vec2::new(x_pos as f32, y_pos as f32)
        }).collect())
    }

}

impl Drawable for Object {
    fn pos(&self) -> Vec2 {
        self.pos
    }

    fn size(&self) -> Vec2 {
         self.size
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
        let room = &self.rooms[self.current_room_index];
        room.background_objects.iter().chain(room.collidable_objects.iter()).for_each(|o| o.draw());

    }
    

}