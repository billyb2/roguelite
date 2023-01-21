use macroquad::prelude::*;

use crate::draw::Drawable;

pub fn get_angle(c: Vec2, e: Vec2) -> f32 {
	let d = c - e;
	d.y.atan2(d.x)
}

enum Orientation {
	Colinear,
	Clockwise,
	CounterClockwise,
}

#[derive(Copy, Clone)]
struct Line {
	point1: Vec2,
	point2: Vec2,
}

fn ccw(point1: Vec2, point2: Vec2, point3: Vec2) -> bool {
	let diff2 = point2 - point1;
	let diff3 = point3 - point1;

	diff3.y * diff2.x > diff2.y * diff3.x
}

impl Line {
	fn intersect(&self, line2: &Line) -> bool {
		ccw(self.point1, line2.point1, line2.point2) != ccw(self.point2, line2.point1, line2.point2) &&
			ccw(self.point1, self.point2, line2.point1) !=
				ccw(self.point1, self.point2, line2.point2)
	}
}

impl Line {
	fn new(point1: Vec2, point2: Vec2) -> Self { Self { point1, point2 } }
}

#[derive(Clone, Copy)]
pub struct Polygon {
	center: Vec2,
	lines: [Line; 4],
}

impl Polygon {
	fn shift(&mut self, dir: Vec2) {
		self.center += dir;
		self.lines.iter_mut().for_each(|line| {
			line.point1 += dir;
			line.point2 += dir;
		});
	}
}

impl Drawable for Polygon {
	fn size(&self) -> Vec2 { Vec2::ZERO }

	fn pos(&self) -> Vec2 { Vec2::ZERO }

	fn draw(&self) {
		self.lines.iter().for_each(|line| {
			draw_line(
				line.point1.x,
				line.point1.y,
				line.point2.x,
				line.point2.y,
				1.0,
				WHITE,
			);
		});
	}
}

impl AsPolygon for Polygon {
	fn as_polygon(&self) -> Polygon { *self }
}

pub trait AsPolygon {
	fn as_polygon(&self) -> Polygon;

	fn center(&self) -> Vec2 { self.as_polygon().center }
}

pub fn easy_polygon(center: Vec2, half_size: Vec2, rotation: f32) -> Polygon {
	let rotated_half_size_x_cos = half_size.x * rotation.cos();
	let rotated_half_size_x_sin = half_size.x * rotation.sin();

	let rotated_half_size_y_cos = half_size.y * rotation.cos();
	let rotated_half_size_y_sin = half_size.y * rotation.sin();

	let corner1 = Vec2::new(
		center.x - rotated_half_size_x_cos - rotated_half_size_y_sin,
		center.y - rotated_half_size_x_sin + rotated_half_size_y_cos,
	);

	let corner2 = Vec2::new(
		center.x + rotated_half_size_x_cos - rotated_half_size_y_sin,
		center.y + rotated_half_size_x_sin + rotated_half_size_y_cos,
	);

	let corner3 = Vec2::new(
		center.x - rotated_half_size_x_cos + rotated_half_size_y_sin,
		center.y - rotated_half_size_x_sin - rotated_half_size_y_cos,
	);

	let corner4 = Vec2::new(
		center.x + rotated_half_size_x_cos + rotated_half_size_y_sin,
		center.y + rotated_half_size_x_sin - rotated_half_size_y_cos,
	);

	let line1 = Line::new(corner1, corner2);
	let line2 = Line::new(corner3, corner4);
	let line3 = Line::new(corner2, corner4);
	let line4 = Line::new(corner1, corner3);

	Polygon {
		center,
		lines: [line1, line2, line3, line4],
	}
}

pub fn point_in_polygon<A: AsPolygon>(polygon: &A, point: Vec2) -> bool {
	let polygon = polygon.as_polygon();
	let point_line = Line::new(point, point + Vec2::new(1000.0, 0.0));

	let num_intersections = polygon
		.lines
		.iter()
		.filter(|line| line.intersect(&point_line))
		.count();

	// If the number of intersections is odd, then the point is inside the polygon
	num_intersections & 1 != 0
}

pub fn aabb_collision<A: AsPolygon, B: AsPolygon>(poly1: &A, poly2: &B, distance: Vec2) -> bool {
	let mut poly1: Polygon = poly1.as_polygon();
	poly1.shift(distance);
	let poly2: Polygon = poly2.as_polygon();

	poly1
		.lines
		.iter()
		.any(|line1| poly2.lines.iter().any(|line2| line1.intersect(line2)))
}

pub fn aabb_collision_dir<A: AsPolygon, B: AsPolygon>(
	aabb1: &A, aabb2: &B, distance: Vec2,
) -> BVec2 {
	let obj1 = aabb1.as_polygon();
	let obj2 = aabb2.as_polygon();

	let mut obj1_pos_x = obj1.clone();
	let mut obj1_pos_y = obj1.clone();

	obj1_pos_x.shift(Vec2::new(distance.x, 0.0));
	obj1_pos_y.shift(Vec2::new(0.0, distance.y));

	let check_collision = |obj: Polygon| -> bool { aabb_collision(&obj, &obj2, distance) };

	BVec2::new(check_collision(obj1_pos_x), check_collision(obj1_pos_y))
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

pub fn fletcher16(data: Vec<u8>) -> u16 {
	let (sum1, sum2) = data.into_iter().fold((0, 0), |(sum1, sum2), byte| {
		let sum1 = (sum1 + byte as u16) % 255;
		let sum2 = (sum2 + sum1) % 255;

		(sum1, sum2)
	});

	(sum2 << 8) | sum1
}
