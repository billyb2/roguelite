use macroquad::prelude::*;
use crate::{
    draw::Drawable, 
    map::Map, 
    math::{AsAABB, AxisAlignedBoundingBox}, attacks::Attack
};

pub const PLAYER_SIZE: f32 = 12.0;

#[derive(Copy, Clone)]
pub enum PlayerClass {
    Warrior,

}

pub struct Player {
    class: PlayerClass,
    pub angle: f32,
    pos: Vec2,
    speed: f32,
    health: f32,
    invincibility_frames: u16, 
    pub attack_cooldown: u16,
    pub attacks: Vec<Box<dyn Attack>>,

}

impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            angle: 0.0,
            class: PlayerClass::Warrior,
            speed: 2.2,
            attack_cooldown: 0,
            health: 100.0,
            invincibility_frames: 0,
            attacks: Vec::with_capacity(2),
        }
    }

    #[inline]
    pub fn pos(&self) -> Vec2 {
        self.pos

    }

    #[inline]
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

    fn draw(&self) {
        draw_rectangle(self.pos.x, self.pos.y, PLAYER_SIZE, PLAYER_SIZE, RED);
        draw_text(&self.health.to_string(), self.pos.x, self.pos.y - PLAYER_SIZE, 12.0, WHITE);
        
    }

}

pub fn move_player(player: &mut Player, angle: f32, speed: Option<Vec2>, map: &Map) {
    let direction: Vec2 = (angle.cos(), angle.sin()).into();
    let distance = direction * speed.unwrap_or_else(|| Vec2::splat(player.speed));

    if !map.collision(player, distance) {
        player.pos += distance;

    }

}

pub fn damage_player(player: &mut Player, damage: f32, damage_direction: f32, map: &Map) {
    if player.invincibility_frames > 0 {
        return;

    }

    let new_health = player.health - damage;

    player.health = match new_health > 0.0 {
        true => new_health,
        false => 0.0,

    };

    // Have the player "flinch" away from damage
    move_player(player, damage_direction, Some(Vec2::splat(PLAYER_SIZE)), map);

    player.invincibility_frames = (damage as u16) * 2;

}

pub fn update_cooldowns(players: &mut [Player]) {
    players.iter_mut().for_each(|p|  {
        p.attack_cooldown = p.attack_cooldown.saturating_sub(1);
        p.invincibility_frames = p.invincibility_frames.saturating_sub(1);

    });

}
