use std::collections::HashMap;

use crate::{
    player::{Player, damage_player},
    monsters::Monster,
    math::{AsAABB, AxisAlignedBoundingBox, get_angle, aabb_collision}, 
    draw::Drawable, map::{TILE_SIZE, Floor},
};
use macroquad::{prelude::*, rand::ChooseRandom};

enum AttackMode {
    Passive,
    Attacking,

}

#[derive(Copy, Clone)]
enum Target {
    Pos(Vec2),
    PlayerIndex(usize),

}

impl Target {
    fn unwrap_pos(self) -> Vec2 {
        match self {
            Self::Pos(v) => v,
            _ => panic!(),

        }
    }
}

const SIZE: f32 = 15.0;
const MAX_HEALTH: f32 = 35.0;

pub struct SmallRat {
    health: f32,
    pos: Vec2,
    texture: Texture2D,
    attack_mode: AttackMode,
    time_spent_moving: u16,
    time_til_move: u16,
    current_path: Option<(Vec<Vec2>, usize)>,
    // Gotta keep track of if the target moved, to reset the path
    current_target: Option<Target>,

}

impl Monster for SmallRat {
    fn new(textures: &HashMap<String, Texture2D>, floor: &Floor) -> Self {
        // Pick all points at least 15 tiles away from all players
        let pos = *floor.background_objects().iter().filter_map(|o| {
            match o.pos().distance(floor.current_spawn()) > (12 * TILE_SIZE) as f32 {
                true =>Some(o.pos()),
                false => None,
            } 
        }).collect::<Vec<Vec2>>().choose().unwrap();

        Self {
            pos,
            health: MAX_HEALTH,
            texture: *textures.get("generic_monster.webp").unwrap(),
            attack_mode: AttackMode::Passive,
            time_til_move: rand::gen_range(0_u32, 180).try_into().unwrap(),
            time_spent_moving: 0,
            current_path: None,
            current_target: None,

        }

    }

    fn movement(&mut self, players: &[Player], map: &Floor) {
        match self.attack_mode {
            AttackMode::Passive => passive_mode(self, players, map),
            AttackMode::Attacking => attack_mode(self, players, map),

        };

    }

    fn damage_players(&mut self, players: &mut [Player], floor: &Floor) {
        players.iter_mut().for_each(|p| {
            if aabb_collision(p, self, Vec2::ZERO) {
                const DAMAGE: f32 = 20.0;
                let damage_direction = get_angle(p.pos().x, p.pos().y, self.pos.x, self.pos.y);

                damage_player(p, DAMAGE, damage_direction, floor);

            }

        });
    }

    fn take_damage(&mut self, damage: f32, damage_direction: f32, _floor: &Floor) {
        self.health -= damage;

        if self.health < 0.0 {
            self.health = 0.0;

        }

        // "Flinch" away from damage
        self.pos += Vec2::new(damage_direction.cos(), damage_direction.sin()) * self.size() * (damage / (MAX_HEALTH / 2.0));

    }

    fn living(&self) -> bool {
        self.health > 0.0
    }


}


const AGGRO_DISTANCE: f32 = (TILE_SIZE * 6) as f32;

// The rat just wanders around a lil in passive mode
fn passive_mode(my_monster: &mut SmallRat, players: &[Player], floor: &Floor) {
    my_monster.time_til_move = my_monster.time_til_move.saturating_sub(1);

    let find_target = || -> Vec2 {
        *floor.background_objects().iter().filter_map(|o| {
            let obj_distance = o.pos().distance(my_monster.pos);

            match obj_distance > (TILE_SIZE * 5) as f32 && obj_distance < (TILE_SIZE * 8) as f32 {
                true => Some(o.pos()),
                false => None,
            }

        }).collect::<Vec<Vec2>>().choose().unwrap()
    };

    if my_monster.current_target.is_none() {
        my_monster.current_target = Some(Target::Pos(find_target()));

    }


    if my_monster.time_til_move == 0 {
        if my_monster.current_path.is_none() {
            if let Some(path) = floor.find_path(my_monster, my_monster.current_target.as_ref().unwrap().unwrap_pos()) {
                my_monster.current_path = Some((path, 1));

            } else {
                my_monster.current_target = Some(Target::Pos(find_target()));


            }


        }

        if let Some((path, i)) = &mut my_monster.current_path {
            if let Some(pos) = path.get(*i) {
                if my_monster.pos.distance(*pos) < 2.0 {
                    *i += 1;

                } else {
                    const PASSIVE_SPEED: f32 = 0.7;

                    let angle = get_angle(pos.x, pos.y, my_monster.pos.x, my_monster.pos.y);
                    my_monster.pos += Vec2::new(angle.cos(), angle.sin()) * PASSIVE_SPEED;

                }


            } else {
                my_monster.current_path = None; 
                my_monster.current_target = None;
                my_monster.time_til_move = rand::gen_range(120_u32, 240).try_into().unwrap();

            }

        }

    }

    // If the rat gets within a few tiles of a player, it'll start attack mode

    if let Some((i, _)) = players.iter().enumerate().find(|(_, p)| p.pos().distance(my_monster.pos) <= AGGRO_DISTANCE && p.health() > 0.0) {
        my_monster.time_til_move = 30;
        my_monster.time_spent_moving = 0;

        my_monster.attack_mode = AttackMode::Attacking;
        my_monster.current_target = Some(Target::PlayerIndex(i));

    }
    

}


fn attack_mode(my_monster: &mut SmallRat, players: &[Player], floor: &Floor) {
    if let Some(Target::PlayerIndex(i)) = my_monster.current_target {
        my_monster.time_til_move = my_monster.time_til_move.saturating_sub(1);

        if my_monster.time_til_move > 0 {
            return;

        }

        let mut target_player = &players[i];

        // First, check the targeted player is still within aggro distance
        let mut distance_from_target = target_player.pos().distance(my_monster.pos);
        let target_in_aggro_range = distance_from_target <= AGGRO_DISTANCE && target_player.health() > 0.0;

        // If it isn't, try to see if there's anohter player within aggro range
        if !target_in_aggro_range {
            if let Some((i, p, distance)) = players.iter().enumerate().find_map(|(_, p)| {
                let distance_from_target = p.pos().distance(my_monster.pos);

                match distance_from_target <= AGGRO_DISTANCE {
                    true => Some((i, p, distance_from_target)),
                    false => None,

                }

            }) {
                my_monster.current_target = Some(Target::PlayerIndex(i));
                target_player = p;
                distance_from_target = distance;

            } else {
                // No players were found in range, just return to passive mode
                my_monster.current_target = None;
                my_monster.attack_mode = AttackMode::Passive;
                return;

            }

        }

        // We now have a player to target, so find the quickest path to get to them
        if my_monster.current_path.is_none() {
            if let Some(path) = floor.find_path(my_monster, target_player.pos()) {
                my_monster.current_path = Some((path, 1));

            }

        }

        if let Some((path, i)) = &mut my_monster.current_path {
            if let Some(pos) = path.get(*i) {
                if my_monster.pos.distance(*pos) < 2.0 {
                    *i += 1;

                } else {
                    let angle = get_angle(pos.x, pos.y, my_monster.pos.x, my_monster.pos.y);
                    my_monster.pos += Vec2::new(angle.cos(), angle.sin());

                }


            } else if let Some(path) = floor.find_path(my_monster, target_player.pos()) {
                my_monster.current_path = Some((path, 1));

            }

        }

        // If the player dies, go back to passive mode
        if target_player.health() == 0.0 {
            my_monster.attack_mode = AttackMode::Passive;
            my_monster.current_target = None;

        }

        // When the monster's within range of the player, "lunge" at them
        if distance_from_target <= TILE_SIZE as f32 {
            let angle = get_angle(target_player.pos().x, target_player.pos().y, my_monster.pos.x, my_monster.pos.y);
            my_monster.pos += Vec2::new(angle.cos(), angle.sin()) * SIZE;
            my_monster.time_til_move = 45;

        }

    }
    
}

impl AsAABB for SmallRat {
    fn as_aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox {
            pos: self.pos,
            size: self.size(),

        }
    }

}


impl Drawable for SmallRat {
    fn pos(&self) -> Vec2 {
        self.pos
        
    }

    fn size(&self) -> Vec2 {
        match self.attack_mode {
            AttackMode::Attacking => Vec2::splat(SIZE * 1.1),
            _ => Vec2::splat(SIZE),

        }
    }

    fn texture(&self) -> Option<Texture2D> {
        Some(self.texture)
        
    }

}
