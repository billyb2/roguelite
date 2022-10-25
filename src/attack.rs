use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::{player::{PlayerClass, Player}, draw::Drawable};

pub struct Attack {
    class: PlayerClass,
    time: u16,
    pos: Vec2,
    angle: f32,
    size: Vec2,
    texture: Option<Texture2D>,

}

impl Attack {
    pub fn new(player: &Player, angle: f32, texture: Option<Texture2D>) -> Self {
        Self {
            class: player.class(),
            time: 0,
            pos: player.pos(),
            angle,
            size: Vec2::splat(50.0),
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

pub fn player_attack(player: &mut Player, attacks: &mut Vec<Attack>, angle: f32, texture: Option<Texture2D>) {
    if player.attack_cooldown != 0 {
        return;

    }

    attacks.push(Attack::new(player, angle, texture));

    player.attack_cooldown = match player.class() {
        PlayerClass::Warrior => 30,

    }

}

pub fn update_attacks(attacks: &mut Vec<Attack>) {
    attacks.drain_filter(|attack| {
        attack.time += 1;

        match attack.class {
            PlayerClass::Warrior => {
                let direction: Vec2 = Vec2::new(attack.angle.cos(), attack.angle.sin());
                attack.pos += direction * 12.0;
                
                // Remove any warrior attacks after X frames
                attack.time >= 8
            },

        }

    });

}
