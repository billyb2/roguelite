use std::f32::consts::{PI, FRAC_PI_2};
use macroquad::prelude::*;

use crate::draw::Drawable;

pub fn get_angle(cx: f32, cy: f32, ex: f32, ey: f32) -> f32 {
    let dy = ey - cy;
    let dx = ex - cx;

    let angle = match dx != 0.0 {
        // Returns the angle in radians
        true => (dy / dx).atan(),
        false => match dy > 0.0 {
            true => FRAC_PI_2, 
            false => FRAC_PI_2,
        },
    };

    match cx < ex {
        true => angle - PI,
        false => angle,

    }
    
}

pub struct AxisAlignedBoundingBox {
    pub pos: Vec2,
    pub size: Vec2,

}

impl Drawable for AxisAlignedBoundingBox {
    fn pos(&self) -> Vec2 {
        self.pos
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn draw(&self) {
        draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, Color::from_rgba(255, 0, 0, 100));
        
    }

}

pub trait AsAABB {
    fn as_aabb(&self) -> AxisAlignedBoundingBox;
}

pub fn aabb_collision(aabb1: &dyn AsAABB, aabb2: &dyn AsAABB, distance: Vec2) -> bool {
    const TWO: Vec2 = Vec2::splat(2.0);

    let mut obj1 = aabb1.as_aabb();
    let obj2 = aabb2.as_aabb();

    obj1.pos += distance;

    let half_obj1_size = obj1.size / TWO;
    let half_obj2_size = obj2.size / TWO;

    let obj1_min = obj1.pos - half_obj1_size;
    let obj1_max = obj1.pos + half_obj1_size;

    let obj2_min = obj2.pos - half_obj2_size;
    let obj2_max = obj2.pos + half_obj2_size;

    obj1_min.cmple(obj2_max).all() &&
    obj2_min.cmple(obj1_max).all()

}
