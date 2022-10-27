use macroquad::prelude::*;
use crate::{
    draw::Drawable, 
    map::Map, 
    math::{AsAABB, AxisAlignedBoundingBox}
};

pub const PLAYER_SIZE: f32 = 12.0;

#[derive(Copy, Clone)]
pub enum PlayerClass {
    Warrior,

}

pub struct Player {
    class: PlayerClass,
    pos: Vec2,
    speed: f32,
    pub attack_cooldown: u16,

}

impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            class: PlayerClass::Warrior,
            speed: 2.2,
            attack_cooldown: 0,
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos

    }

    pub fn class(&self) -> PlayerClass {
        self.class

    }

}

impl AsAABB for Player {
    fn as_aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox {
            pos: self.pos,
            size: Vec2::splat(PLAYER_SIZE),

        }
        
    }

}

impl Drawable for Player {
    fn pos(&self) -> Vec2 {
        self.pos

    }

    fn size(&self) -> Vec2 {
        Vec2::splat(PLAYER_SIZE)

    }

}

pub fn move_player(player: &mut Player, angle: f32, map: &Map) {
    let direction: Vec2 = (angle.cos(), angle.sin()).into();
    let distance = direction * player.speed;

    if !map.collision(player, distance) {
        player.pos += distance;

    }

}

pub fn update_cooldowns(players: &mut [Player]) {
    players.iter_mut().for_each(|p| 
        p.attack_cooldown = p.attack_cooldown.saturating_sub(1)

    );

}
