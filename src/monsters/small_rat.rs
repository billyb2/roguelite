use std::collections::HashMap;

use crate::{
    player::Player,
    monsters::Monster,
    math::{AsAABB, AxisAlignedBoundingBox, get_angle}, 
    draw::Drawable, map::{Map, TILE_SIZE},
};
use macroquad::{prelude::*, rand::ChooseRandom};

enum AttackMode {
    Passive,

}

pub struct SmallRat {
    pos: Vec2,
    size: Vec2,
    texture: Texture2D,
    attack_mode: AttackMode,
    time_spent_wandering: u16,
    time_til_wander: u16,
    current_path: Option<(Vec<Vec2>, usize)>,
    // Gotta keep track of if the target moved, to reset the path
    current_target: Option<Vec2>,

}

impl Monster for SmallRat {
    fn new(textures: &HashMap<String, Texture2D>, map: &Map) -> Self {
        // Pick all points at least 15 tiles away from all players
        let pos = *map.current_room().background_objects().iter().filter_map(|o| {
            if o.pos().distance(map.current_spawn()) > (12 * TILE_SIZE) as f32 {
                Some(o.pos())

            } else {
                None

            }
        }).collect::<Vec<Vec2>>().choose().unwrap();
        


        Self {
            pos,
            size: Vec2::splat(15.0),
            texture: *textures.get("generic_monster.webp").unwrap(),
            attack_mode: AttackMode::Passive,
            // 3.0 seconds at 60 fps
            time_til_wander: 0,
            time_spent_wandering: 0,
            current_path: None,
            current_target: None,

        }

    }

    fn ai(&mut self, players: &mut [Player], map: &Map) {
        match self.attack_mode {
            AttackMode::Passive => passive_mode(self, players, map),

        }

    }

}

// The rat just wanders around a lil in passive mode
fn passive_mode(my_monster: &mut SmallRat, players: &mut [Player], map: &Map) {
    my_monster.time_til_wander = my_monster.time_til_wander.saturating_sub(1);

    let find_target = || -> Vec2 {
        *map.current_room().background_objects().iter().filter_map(|o| {
            let obj_distance = o.pos().distance(my_monster.pos);

            if obj_distance > (TILE_SIZE * 5) as f32 && obj_distance < (TILE_SIZE * 8) as f32 {
                Some(o.pos())

            } else {
                None

            }
        }).collect::<Vec<Vec2>>().choose().unwrap()
    };

    if my_monster.current_target.is_none() {
        my_monster.current_target = Some(find_target());

    }


    if my_monster.time_til_wander == 0 {
        if my_monster.current_path.is_none() {
            if let Some(path) = map.find_path(my_monster.pos, my_monster.current_target.unwrap()) {
                my_monster.current_path = Some((path, 1));

            } else {
                my_monster.current_target = Some(find_target());


            }


        }

        if let Some((path, i)) = &mut my_monster.current_path {
            if let Some(pos) = path.get(*i) {
                if my_monster.pos.distance(*pos) < 2.0 {
                    *i += 1;

                } else {
                    let angle = get_angle(pos.x, pos.y, my_monster.pos.x, my_monster.pos.y);
                    my_monster.pos += Vec2::new(angle.cos(), angle.sin()) * 0.7;

                }


            } else {
                my_monster.current_path = None; 
                my_monster.current_target = None;
                my_monster.time_til_wander = rand::gen_range(120_u32, 240).try_into().unwrap();

            }

        }

    }
    

}

impl AsAABB for SmallRat {
    fn as_aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox {
            pos: self.pos,
            size: self.size,

        }
    }

}


impl Drawable for SmallRat {
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
