use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use macroquad::miniquad::fs;
use macroquad::prelude::*;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

pub type Textures = Lazy<HashMap<String, Texture2D>>;

pub static TEXTURES: Textures = Lazy::new(|| {
	let textures = Arc::new(Mutex::new(HashMap::new()));

	for asset_name in ASSETS {
		let path = format!("assets/{asset_name}");
		let textures = textures.clone();

		fs::load_file(&path, move |bytes| {
			let texture =
				Texture2D::from_file_with_format(&bytes.unwrap(), Some(ImageFormat::WebP));
			textures
				.lock()
				.unwrap()
				.insert(asset_name.to_string(), texture);
		});
	}

	let textures = textures.lock().unwrap();

	textures.clone()
});

pub fn load_my_image(image_name: &str) -> Texture2D { *TEXTURES.get(image_name).unwrap() }

/*
pub fn load_my_image(image_name: &str) -> Texture2D {
	let textures = TEXTURES.read().unwrap();

	if let Some(texture) = textures.get(image_name) {
		return *texture;
	}

	std::mem::drop(textures);
;
	let textures = &mut TEXTURES.write().unwrap();

	let path = format!("assets/{image_name}");
	let texture = block_on(load_texture(&path)).unwrap();

	textures.insert(image_name.to_string(), texture.clone());

	texture
}
*/

pub trait Drawable {
	fn size(&self) -> Vec2;
	fn pos(&self) -> Vec2;
	fn rotation(&self) -> f32 { 0.0 }
	fn texture(&self) -> Option<Texture2D> { None }
	fn flip_x(&self) -> bool { true }
	fn draw(&self) {
		let size = self.size();
		let pos = self.pos();

		match self.texture() {
			Some(texture) => {
				let texture_params = DrawTextureParams {
					rotation: self.rotation(),
					flip_x: self.flip_x(),
					dest_size: Some(size),
					..Default::default()
				};

				draw_texture_ex(texture, pos.x, pos.y, WHITE, texture_params);
			},
			None => draw_rectangle(pos.x, pos.y, size.x, size.y, RED),
		};
	}
}
