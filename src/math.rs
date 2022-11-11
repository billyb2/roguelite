use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

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

#[derive(Clone)]
pub struct AxisAlignedBoundingBox {
	pub pos: Vec2,
	pub size: Vec2,
}

impl AsAABB for AxisAlignedBoundingBox {
	fn as_aabb(&self) -> AxisAlignedBoundingBox {
		self.clone()
	}
}

impl Drawable for AxisAlignedBoundingBox {
	fn pos(&self) -> Vec2 {
		self.pos
	}

	fn size(&self) -> Vec2 {
		self.size
	}

	fn draw(&self) {
		draw_rectangle(
			self.pos.x,
			self.pos.y,
			self.size.x,
			self.size.y,
			Color::from_rgba(255, 0, 0, 100),
		);
	}
}

pub trait AsAABB {
	fn as_aabb(&self) -> AxisAlignedBoundingBox;

	fn center(&self) -> Vec2 {
		let aabb = self.as_aabb();
		aabb.pos + (aabb.size / 2.0)
	}
}

pub fn aabb_collision<A: AsAABB, B: AsAABB>(aabb1: &A, aabb2: &B, distance: Vec2) -> bool {
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

	obj1_min.cmple(obj2_max).all() && obj2_min.cmple(obj1_max).all()
}

/// Bresenhams Circle Algorithm
pub fn points_on_circumference(center: IVec2, radius: i32) -> Vec<IVec2> {
	// Distance from center
	let mut d = IVec2::new(radius, 0);
	let mut o2 = 1 - radius;

	// TODO: Use an ArrayVec
	let mut points = Vec::new();

	while d.y <= d.x {
		points.push(center + d);
		points.push(center + d.yx());
		points.push(center + (d * IVec2::new(-1, 1)));
		points.push(center + (d.yx() * IVec2::new(-1, 1)));
		points.push(center + (d * IVec2::new(-1, -1)));
		points.push(center + (d.yx() * IVec2::new(-1, -1)));
		points.push(center + (d * IVec2::new(1, -1)));
		points.push(center + (d.yx() * IVec2::new(1, -1)));

		d.y += 1;

		if o2 <= 0 {
			o2 += (2 * d.y) + 1;
		} else {
			d.x -= 1;
			o2 += (2 * (d.y - d.x)) + 1;
		}
	}

	points
}

pub fn points_on_line(pos1: IVec2, pos2: IVec2) -> Vec<IVec2> {
	let mut d = (pos2 - pos1).abs();

	let mut pos = pos1;
	let mut n = 1 + d.x + d.y;

	let inc = -(pos1 - pos2).signum();

	let mut err = d.x - d.y;

	d *= 2;

	let mut lines = Vec::new();

	while n > 0 {
		lines.push(pos);

		if err > 0 {
			pos.x += inc.x;
			err -= d.y;
		} else {
			pos.y += inc.y;
			err += d.x;
		}

		n -= 1;
	}

	lines
}
