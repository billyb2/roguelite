use macroquad::prelude::*;

use crate::{player::{PlayerClass, Player}, draw::Drawable, map::Map, math::{AsAABB, AxisAlignedBoundingBox, aabb_collision}, monsters::Monster};

#[derive(Copy, Clone)]
pub enum AttackType {
    Primary,
    Secondary,

}

pub struct Attack {
    attack_type: AttackType,
    class: PlayerClass,
    time: u16,
    pos: Vec2,
    angle: f32,
    size: Vec2,
    player: usize,
    damage: f32,
    texture: Option<Texture2D>,

}

impl AsAABB for Attack {
    fn as_aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox {
            pos: self.pos,
            size: self.size,

        }
    }

}

impl Drawable for Attack {
    fn pos(&self) -> Vec2 {
        self.pos
        
    }

    fn size(&self) -> Vec2 {
        self.size

    }

    fn rotation(&self) -> f32 {
        self.angle
    }

    fn texture(&self) -> Option<Texture2D> {
        self.texture
        
    }

}

pub fn player_attack(player: &mut Player, player_index: usize, attack_type: AttackType, attacks: &mut Vec<Attack>, angle: f32, texture: Option<Texture2D>) {
    if player.attack_cooldown != 0 {
        return;

    }


    let (size, attack_cooldown, damage) = match player.class() {
        PlayerClass::Warrior => match attack_type {
            AttackType::Primary => (Vec2::new(15.0, 25.0), 20, 5.),
            AttackType::Secondary => (Vec2::new(25.0, 5.0), 45, 25.),

        }
    };

    player.attack_cooldown = attack_cooldown;
    attacks.push(Attack {
        attack_type,
        angle,
        class: player.class(),
        time: 0,
        damage,
        size,
        pos: player.pos(),
        player: player_index,
        texture,


    });

}

pub fn update_attacks(attacks: &mut Vec<Attack>, players: &mut [Player], monsters: &mut [Box<dyn Monster>], map: &Map) {
    attacks.drain_filter(|attack| {
        attack.time += 1;

        match attack.class {
            PlayerClass::Warrior => {
                let direction: Vec2 = Vec2::new(attack.angle.cos(), attack.angle.sin());

                let old_pos = attack.pos;

                let movement = direction * match attack.attack_type {
                    AttackType::Primary => 5.0,
                    AttackType::Secondary => 9.0,

                };
                
                if !map.collision(attack, movement){
                    attack.pos += movement;

                }

                // Find all monsters colliding with an attack, and do damage to them
                monsters.iter_mut().filter(|m| aabb_collision(&m.into_aabb_obj(), attack, Vec2::ZERO)).for_each(|m| {
                    m.take_damage(attack.player, players, attack.damage, map);

                });
    
                // Remove any warrior attacks after X frames
                attack.time >= 10
            },

        }

    });

}
