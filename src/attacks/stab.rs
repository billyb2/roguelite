use std::collections::HashMap;

use crate::{player::Player, draw::Drawable, map::{Map, self}, monsters::Monster, math::{AsAABB, AxisAlignedBoundingBox, aabb_collision, get_angle}};
use macroquad::prelude::*;

use super::Attack;

const SIZE: Vec2 = Vec2::new(25.0, 5.0);

pub struct Stab {
    pos: Vec2,
    angle: f32,
    texture: Texture2D,
    time: u16,

}

impl Attack for Stab {
    fn new(player: &mut Player, angle: f32, textures: &HashMap<String, Texture2D>, _map: &Map) -> Box<Self> {
        Box::new(Self {
            pos: player.pos(),
            angle,
            texture: *textures.get("stab.webp").unwrap(),
            time: 0,

        })
    }

    fn update(&mut self, monsters: &mut [Box<dyn Monster>], map: &map::Map) -> bool {
        let movement = Vec2::new(self.angle.cos(), self.angle.sin()) * 9.0;

        if !map.collision(self, movement) {
            self.pos += movement;
            self.time += 1;

        } else {
            return true;

        } 

        if self.time >= 10 {
            return true;

        }

        // Check to see if it's collided with a monster
        if let Some(monster) = monsters.iter_mut().find(|m| aabb_collision(self, &m.as_aabb(), Vec2::ZERO)) {
            const DAMAGE: f32 = 25.0;

            let damage_direction = get_angle(monster.pos().x, monster.pos().y, self.pos.x, self.pos.y);
            monster.take_damage(DAMAGE, damage_direction, map);

            return true;

        }

        false

    }

    fn cooldown(&self) -> u16 {
        45
    }

}

impl AsAABB for Stab {
    fn as_aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox {
            pos: self.pos, 
            size: SIZE,
        }
    }

}

impl Drawable for Stab {
    fn pos(&self) -> Vec2 {
        self.pos

    }

    fn size(&self) -> Vec2 {
        SIZE
    }

    fn rotation(&self) -> f32 {
        self.angle
    }

    fn texture(&self) -> Option<Texture2D> {
        Some(self.texture)
        
    }

}

