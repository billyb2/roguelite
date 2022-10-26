use std::collections::HashMap;

use macroquad::prelude::*;
use here_be_dragons::*;

use crate::{
    draw::Drawable, 
    math::{AsAABB, aabb_collision, AxisAlignedBoundingBox}
};

struct Object {
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

struct Room {
    spawn: Vec2,
    objects: Vec<Object>,
    exit_indices: Vec<usize>,

}

pub struct Map {
    current_room_index: usize,
    rooms: Vec<Room>,

}

impl Map {
    pub fn new(textures: &HashMap<String, Texture2D>) -> Self {
        let tile_w = 30;
        let tile_h = 30;

        let map_gen = MapBuilder::<NoData>::new(tile_w, tile_h)
            .with(NoiseGenerator::new(0.1))
            .with(BspRooms::new())
            .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
            .with(DistantExit::new())
            .build();

        let size = 20;

        let mut objects: Vec<Object> = map_gen.tiles.iter().enumerate().filter_map(|(i, tile)| {
            if tile.is_blocked() {
                let x_pos = (i / tile_w) * size;
                let y_pos = (i % tile_h) * size;

                Some(Object {
                    pos: Vec2::new(x_pos as f32, y_pos as f32),
                    size: Vec2::splat(size as f32),
                    texture: *textures.get("sad_face.webp").unwrap(),
                    is_exit: false,
                })

            } else {
                None

            }
            
        }).collect(); 

        objects.push(Object {
            pos: map_gen.exit_point.map(|p| Vec2::new(p.x as f32, p.y as f32) * Vec2::splat(size as f32)).unwrap(),
            size: Vec2::splat(size as f32),
            texture: *textures.get("green.webp").unwrap(),
            is_exit: true,

        });

        let rooms = vec![Room {
            spawn: map_gen.starting_point.map(|p| Vec2::new(p.x as f32, p.y as f32) * Vec2::splat(size as f32)).unwrap(),
            objects,
            exit_indices: Vec::new(),

        }];
        Self {
            current_room_index: 0,
            rooms,

        }

    }

    pub fn current_spawn(&self) -> Vec2 {
        self.rooms[self.current_room_index].spawn

    }

    pub fn collision(&self, aabb: &dyn AsAABB, distance: Vec2) -> bool {
        let current_room = &self.rooms[self.current_room_index];

        current_room.objects.iter().any(|object| 
            if !object.is_exit {
                aabb_collision(aabb, object, distance)

            } else {
                false

            }

        )

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
        room.objects.iter().for_each(|o| o.draw());
        room.objects.iter().for_each(|o| o.as_aabb().draw());

    }
    

}
