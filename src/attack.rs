use macroquad::prelude::*;

use crate::{player::{PlayerClass, Player}, draw::Drawable};

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
    texture: Option<Texture2D>,

}

impl Attack {
    pub fn new(player: &Player, attack_type: AttackType, angle: f32, size: Vec2, texture: Option<Texture2D>) -> Self {
        Self {
            attack_type,
            class: player.class(),
            time: 0,
            pos: player.pos(),
            angle,
            size,
            texture,
            
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

pub fn player_attack(player: &mut Player, attack_type: AttackType, attacks: &mut Vec<Attack>, angle: f32, texture: Option<Texture2D>) {
    if player.attack_cooldown != 0 {
        return;

    }


    let (size, attack_cooldown) = match player.class() {
        PlayerClass::Warrior => match attack_type {
            AttackType::Primary => (Vec2::new(15.0, 25.0), 20),
            AttackType::Secondary => (Vec2::new(25.0, 5.0), 45),

        }
    };

    player.attack_cooldown = attack_cooldown;
    attacks.push(Attack::new(player, attack_type, angle, size, texture));

}

pub fn update_attacks(attacks: &mut Vec<Attack>) {
    attacks.drain_filter(|attack| {
        attack.time += 1;

        match attack.class {
            PlayerClass::Warrior => {
                let direction: Vec2 = Vec2::new(attack.angle.cos(), attack.angle.sin());
                attack.pos += direction * match attack.attack_type {
                    AttackType::Primary => 5.0,
                    AttackType::Secondary => 9.0,

                };
                
                // Remove any warrior attacks after X frames
                attack.time >= 10
            },

        }

    });

}
