use macroquad::prelude::*;



pub trait Drawable {
	fn size(&self) -> Vec2;
	fn pos(&self) -> Vec2;
	fn rotation(&self) -> f32 {
		0.0
	}
	fn texture(&self) -> Option<Texture2D> {
		None
	}
	fn flip_x(&self) -> bool {
		true
	}
	fn draw(&self) {
		let size = self.size();
		let pos = self.pos() - (size * Vec2::splat(0.25));

		match self.texture() {
			Some(texture) => {
				let texture_params = DrawTextureParams {
					rotation: self.rotation(),
					flip_x: self.flip_x(),
					dest_size: Some(size),
					..Default::default()
				};

				draw_texture_ex(texture, pos.x, pos.y, WHITE, texture_params)
			},
			None => draw_rectangle(pos.x, pos.y, size.x, size.y, RED),
		};
	}
}
