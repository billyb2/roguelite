use std::collections::HashMap;

use macroquad::{prelude::*, rand::ChooseRandom};
use here_be_dragons::*;
use pathfinding::prelude::*;
use rayon::prelude::*;

use crate::{
    draw::Drawable, 
    math::{AsAABB, aabb_collision, AxisAlignedBoundingBox}, monsters::{Monster, SmallRat}, player::{PLAYER_SIZE, Player}};

pub const TILE_SIZE: usize = 25;
pub const MAP_WIDTH_TILES: usize = 50;
pub const MAP_HEIGHT_TILES: usize = 50;

#[derive(Clone)]
pub struct Object {
    pos: IVec2,
    texture: Texture2D,

}

impl AsAABB for Object {
    fn as_aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox { 
            pos: self.pos.as_vec2() * Vec2::splat(TILE_SIZE as f32), 
            size: Vec2::splat(TILE_SIZE as f32),
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
        let mut spawn = None;
        let mut exit = None;

        let (mut collidable_objects, mut background_objects) = (Vec::new(), Vec::new());

        // Keep trying to generate a map until it generates a valid one
        while spawn.is_none() {
            collidable_objects.clear();
            background_objects.clear();

            let map_gen = MapBuilder::<NoData>::new(MAP_WIDTH_TILES, MAP_HEIGHT_TILES)
                .with(NoiseGenerator::new(0.50))
                .with(CellularAutomata::new())
                .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
                .with(DistantExit::new())
                .build();

            let tile_to_object = |i: usize, texture: Texture2D| -> Object {
                let x_pos = i % MAP_WIDTH_TILES;
                let y_pos = i / MAP_HEIGHT_TILES;

                Object {
                    pos: IVec2::new(x_pos.try_into().unwrap(), y_pos.try_into().unwrap()),
                    texture,

                }

            };

            let (collidable_tiles, background_tiles): (Vec<_>, Vec<_>) = map_gen.tiles.iter().enumerate().map(|(i, t)| (t.is_blocked(), i)).partition(|(is_blocked, _)| *is_blocked);

            let collidable_object_texture = *textures.get("black.webp").unwrap();
            let background_object_texture = *textures.get("light_gray.webp").unwrap();

            collidable_objects.extend(collidable_tiles.into_iter().map(|(_, i)| tile_to_object(i, collidable_object_texture)));
            background_objects.extend(background_tiles.into_iter().map(|(_, i)| tile_to_object(i, background_object_texture)));

            let exit_point = map_gen.exit_point.unwrap();
            let exit_pos = IVec2::new(exit_point.x.try_into().unwrap(), exit_point.y.try_into().unwrap());

            exit = Some(Object {
                pos: exit_pos,
                texture: *textures.get("green.webp").unwrap(),

            });

            background_objects.shuffle();

            spawn = background_objects.par_iter().find_map_any(|o| {
                // First, find any points greater than half the map
                if distance_squared(o.pos, exit_pos) < ((MAP_WIDTH_TILES * TILE_SIZE) / 2) as i32 {
                    return None;

                } 

                let aabb = AxisAlignedBoundingBox {
                    pos: o.pos.as_vec2() * Vec2::splat(TILE_SIZE as f32) + Vec2::splat(PLAYER_SIZE / 2.0),
                    size: Vec2::splat(PLAYER_SIZE),

                };

                // Then, only keep background objects with a definite path to the exit
                match find_path(&aabb, exit_pos.as_vec2() * Vec2::splat(TILE_SIZE as f32), &collidable_objects).is_some() {
                    true => Some(o.pos.as_vec2() * Vec2::splat(TILE_SIZE as f32)),
                    false => None,
                }

            });

        }

        Floor {
            spawn: spawn.unwrap(),
            collidable_objects,
            background_objects,
            exit: exit.unwrap(),

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
        let room = &self.rooms[self.current_floor_index];
        room.background_objects.iter().chain(room.collidable_objects.iter()).for_each(|o| o.draw());
        room.exit.draw();

    }
    

}

fn find_viable_neighbors(collidable_objects: &[Object], pos: IVec2) -> Vec<(IVec2, i32)> {
    let mut potential_neighbors = {
        let mut positions = [Some(pos); 4];
        let changes = [IVec2::new(-1, 0), IVec2::new(1, 0), IVec2::new(0, -1), IVec2::new(0, 1)];

        positions.iter_mut().zip(changes.into_iter()).for_each(|(p, change)| {
            let new_pos = p.unwrap() + change;

            if new_pos.x < 0 || new_pos.x >= MAP_WIDTH_TILES.try_into().unwrap() || new_pos.y < 0 || new_pos.y >= MAP_HEIGHT_TILES.try_into().unwrap() {
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

    potential_neighbors.iter().filter_map(|p| p.map(|p| (p, 1))).collect()

}

pub fn find_path(start: &dyn AsAABB, goal: Vec2, collidable_objects: &[Object]) -> Option<Vec<Vec2>> {
    let aabb = start.as_aabb();

    let start_tile_pos = (aabb.pos / Vec2::splat(TILE_SIZE as f32)).as_ivec2();
    let goal_tile_pos = (goal / Vec2::splat(TILE_SIZE as f32)).as_ivec2();
    
    let path = astar(
        &start_tile_pos, 
        |pos| find_viable_neighbors(collidable_objects, *pos), 
        |pos| distance_squared(*pos, goal_tile_pos), 
        |pos| *pos == goal_tile_pos,
    );

    path.map(|(positions, _)| positions.iter().map(|pos| {
        (*pos * IVec2::splat(TILE_SIZE as i32)).as_vec2() - (aabb.size * 0.25)
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

fn distance(pos1: IVec2, pos2: IVec2) -> f32 {
    let distance_squared = distance_squared(pos1, pos2);

    (distance_squared as f32).sqrt()

}

fn distance_squared(pos1: IVec2, pos2: IVec2) -> i32 {
    let mut diff = pos2 - pos1;
    diff = diff * diff;

    diff.x + diff.y
}
