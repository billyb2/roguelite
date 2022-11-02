use std::collections::HashMap;

use macroquad::{prelude::*, rand::ChooseRandom};
use here_be_dragons::*;
use ordered_float::OrderedFloat;
use pathfinding::prelude::*;

use crate::{
    draw::Drawable, 
    math::{AsAABB, aabb_collision, AxisAlignedBoundingBox}, monsters::{Monster, SmallRat}, player::{PLAYER_SIZE, Player}};

#[derive(Clone)]
pub struct Object {
    pos: Vec2,
    size: Vec2,
    texture: Texture2D,

}

impl AsAABB for Object {
    fn as_aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox { 
            pos: self.pos, 
            size: self.size,
        }
        
    }

}

pub struct Floor {
    spawn: Vec2,
    collidable_objects: Vec<Object>,
    background_objects: Vec<Object>,
    exit: Object,

}

impl Floor {
    pub fn background_objects(&self) -> &[Object] {
        &self.background_objects

    }

    pub fn collision<A: AsAABB>(&self, aabb: &A, distance: Vec2) -> bool {
        self.collidable_objects.iter().any(|object| 
            aabb_collision(aabb, object, distance)

        )

    }

    pub fn current_spawn(&self) -> Vec2 {
        self.spawn

    }

    pub fn new(_floor_num: usize, textures: &HashMap<String, Texture2D>) -> Self {
        let map_gen = MapBuilder::<NoData>::new(MAP_WIDTH_TILES, MAP_HEIGHT_TILES)
            .with(NoiseGenerator::new(0.55))
            .with(CellularAutomata::new())
            .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
            .with(DistantExit::new())
            .build();

        let tile_to_object = |i: usize, texture: Texture2D| -> Object {
            let x_pos = (i % MAP_WIDTH_TILES) * TILE_SIZE;
            let y_pos = (i / MAP_HEIGHT_TILES) * TILE_SIZE;
            
            Object {
                pos: Vec2::new(x_pos as f32, y_pos as f32),
                size: Vec2::splat(TILE_SIZE as f32),
                texture,

            }

        };

        let (collidable_objects, background_objects): (Vec<_>, Vec<_>) = map_gen.tiles.iter().enumerate().map(|(i, t)| (t.is_blocked(), i)).partition(|(is_blocked, _)| *is_blocked);

        let collidable_object_texture = *textures.get("black.webp").unwrap();
        let background_object_texture = *textures.get("light_gray.webp").unwrap();

        let collidable_objects: Vec<Object> = collidable_objects.into_iter().map(|(_, i)| tile_to_object(i, collidable_object_texture)).collect();
        let mut background_objects: Vec<Object> = background_objects.into_iter().map(|(_, i)| tile_to_object(i, background_object_texture)).collect();


        let exit_pos = map_gen.exit_point.map(|p| Vec2::new(p.x as f32, p.y as f32) * Vec2::splat(TILE_SIZE as f32)).unwrap();

        let exit = Object {
            pos: exit_pos,
            size: Vec2::splat(TILE_SIZE as f32),
            texture: *textures.get("green.webp").unwrap(),

        };

        background_objects.shuffle();

        let spawn = background_objects.iter().find_map(|o| {
            // First, find any points greater than half the map
            if o.pos.distance(exit_pos) < ((MAP_WIDTH_TILES * TILE_SIZE) / 2) as f32 {
                return None;

            } 

            let aabb = AxisAlignedBoundingBox {
                pos: o.pos + Vec2::splat(TILE_SIZE as f32 / 2.0) - Vec2::splat(PLAYER_SIZE / 2.0),
                size: Vec2::splat(PLAYER_SIZE),

            };

            // Then, only keep background objects with a definite path to the exit
            match find_path(&aabb, exit_pos, &collidable_objects).is_some() {
                true => Some(o.pos),
                false => None,
            }

        }).unwrap();

        Floor {
            spawn,
            collidable_objects,
            background_objects,
            exit,

        }

    }

    pub fn find_path(&self, pos: &dyn AsAABB, goal: Vec2) -> Option<Vec<Vec2>> {
        find_path(pos, goal, &self.collidable_objects)

    }

    pub fn should_descend(&self, players: &[Player], _monsters: &[Box<dyn Monster>]) -> bool {
        // If any players are touching the exit, descend a floor
        players.iter().any(|p| {
            aabb_collision(p, &self.exit, Vec2::ZERO)

        })

    }

}

pub struct Map {
    current_floor_index: usize,
    rooms: Vec<Floor>,

}

pub const TILE_SIZE: usize = 25;
pub const MAP_WIDTH_TILES: usize = 50;
pub const MAP_HEIGHT_TILES: usize = 50;

impl Map {
    pub fn new(textures: &HashMap<String, Texture2D>, monsters: &mut Vec<Box<dyn Monster>>) -> Self {
        let floors: Vec<Floor> = (0..5).into_iter().map(|floor_num| {
            Floor::new(floor_num, textures)

        }).collect();

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

    pub fn descend(&mut self, players: &mut [Player], monsters: &mut Vec<Box<dyn Monster>>, textures: &HashMap<String, Texture2D>) {
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
        let room = &self.rooms[self.current_floor_index];
        room.background_objects.iter().chain(room.collidable_objects.iter()).for_each(|o| o.draw());
        room.exit.draw();

    }
    

}

// (X, Y), DISTANCE_BETWEEN
type Viability = ((OrderedFloat<f32>, OrderedFloat<f32>), OrderedFloat<f32>);

fn find_viable_neighbors(collidable_objects: &[Object], size: Vec2, (OrderedFloat(pos_x), OrderedFloat(pos_y)): (OrderedFloat<f32>, OrderedFloat<f32>)) -> Vec<Viability> {
    let pos = Vec2::new(pos_x, pos_y);
    let size_squared = ((size.x + size.y) / 2.0).powi(2);

    [Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, -1.0), Vec2::new(0.0, 1.0)].into_iter().filter_map(|change| {
        let new_pos = pos + change * size;
        let aabb = AxisAlignedBoundingBox {
            pos: new_pos,
            size,
        };

        let collision = collidable_objects.iter().any(|o| {
            aabb_collision(o, &aabb, Vec2::ZERO)
        });

        match collision {
            true => None,
            // Return the center of each object
            false => {
                Some((
                    (OrderedFloat(new_pos.x), OrderedFloat(new_pos.y)), 
                    OrderedFloat(size_squared)
                ))

            },
        }

    }).collect()

}

pub fn find_path(start: &dyn AsAABB, goal: Vec2, collidable_objects: &[Object]) -> Option<Vec<Vec2>> {
    let aabb = start.as_aabb();
    let pos = aabb.pos;
    
    let avg_size = (aabb.size.x + aabb.size.y) / 2.0;

    let path = astar(
        &(OrderedFloat(pos.x), OrderedFloat(pos.y)), 
        |pos| find_viable_neighbors(collidable_objects, aabb.size, *pos), 
        |(OrderedFloat(pos_x), OrderedFloat(pos_y))| OrderedFloat(Vec2::new(*pos_x, *pos_y).distance_squared(goal)), 
        // The goal is reached when we're fairly close to it
        |(OrderedFloat(pos_x), OrderedFloat(pos_y))| Vec2::new(*pos_x, *pos_y).distance(goal) <= avg_size,
    );

    path.map(|(positions, _)| positions.iter().map(|(OrderedFloat(pos_x), OrderedFloat(pos_y))| {
        Vec2::new(*pos_x, *pos_y) + (aabb.size / 2.0)
    }).collect())

}

fn spawn_monsters(_floor_num: usize, monsters: &mut Vec<Box<dyn Monster>>, textures: &HashMap<String, Texture2D>, floor: &Floor) {
    monsters.extend(
        (0..25).map(|_| {
            let monster: Box<dyn Monster> = Box::new(SmallRat::new(textures, floor));
            monster

        })

    );

}
