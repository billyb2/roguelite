mod attacks;
mod draw;
mod enchantments;
mod input;
mod items;
mod map;
mod math;
mod monsters;
mod player;

use std::fs;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use attacks::*;
use draw::*;
use gilrs::Gilrs;
use input::*;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use map::*;
use monsters::*;
use player::*;

use macroquad::miniquad::conf::Platform;
use macroquad::prelude::*;
use macroquad::ui::{root_ui, Skin};
use once_cell::sync::Lazy;

use crate::enchantments::{Enchantable, EnchantmentKind};
use crate::math::AsAABB;

pub const MAX_VIEW_OF_PLAYER: f32 = 200.0;

const DEFAULT_FRAGMENT_SHADER: &str = "
#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
uniform lowp float lowest_light_level;
uniform lowp float window_height;
const lowp float VISION_SIZE = 400.0;

void main() {
    gl_FragColor = texture2D(Texture, uv);

	float lighting = 1.0;
	lighting *= lowest_light_level;
	gl_FragColor.rgb *= vec3(lighting * 0.75);

}
";

const DEFAULT_VERTEX_SHADER: &str = "
#version 100
precision lowp float;
attribute vec3 position;
attribute vec2 texcoord;
varying vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
";

const CAMERA_ZOOM: f32 = 0.0045;

pub static TEXTURES: Lazy<Textures> = Lazy::new(|| {
	fs::read_dir("assets")
		.unwrap()
		.filter_map(|file| {
			if let Ok(file) = file {
				let file_name = file.file_name().to_str().unwrap().to_string();

				let image_bytes = fs::read(file.path()).unwrap();
				let texture = Texture2D::from_file_with_format(&image_bytes, None);

				Some((file_name, texture))
			} else {
				None
			}
		})
		.collect()
});

pub const NUM_PLAYERS: usize = 1;

#[macroquad::main(window_conf)]
async fn main() {
	let time = SystemTime::elapsed(&UNIX_EPOCH).unwrap().as_secs();
	rand::srand(time);
	print!("What player class are you? Warrior, Wizard, or Rogue?: ");
	io::stdout().flush().unwrap();

	let mut class = String::new();
	// io::stdin().read_line(&mut class).unwrap();

	class = class.to_lowercase();
	class.pop();

	let class = match class.is_empty() {
		true => PlayerClass::Wizard,
		false => match class.as_str().try_into() {
			Ok(class) => class,
			Err(_) => panic!("Invalid class given: {class}"),
		},
	};

	let mut attacks = Vec::new();

	let mut map = Map::new(&TEXTURES);

	let mut players: Vec<_> = (0..NUM_PLAYERS)
		.into_iter()
		.map(|i| Player::new(i, class, map.current_floor().current_spawn(), &TEXTURES))
		.collect();

	let mut viewport_screen_height = screen_height() * (1.0 / NUM_PLAYERS as f32);

	let mut cameras: Vec<Camera2D> = players
		.iter()
		.enumerate()
		.map(|(i, p)| Camera2D {
			target: p.center(),
			zoom: Vec2::new(
				CAMERA_ZOOM,
				-CAMERA_ZOOM * (screen_width() / viewport_screen_height),
			) * 0.7,
			viewport: Some((
				0,
				viewport_screen_height as i32 * i as i32,
				screen_width() as i32,
				viewport_screen_height as i32,
			)),
			..Default::default()
		})
		.collect();

	let mut gilrs = Gilrs::new().unwrap();
	let mut active_gamepad = None;

	const SHOW_FRAMERATE: bool = true;

	let mut frames_till_update_framerate: u8 = 30;
	let mut fps = 0.0;

	let fragment_shader = DEFAULT_FRAGMENT_SHADER.to_string();
	let vertex_shader = DEFAULT_VERTEX_SHADER.to_string();

	let pipeline_params = PipelineParams {
		depth_write: true,
		depth_test: Comparison::LessOrEqual,
		color_blend: Some(BlendState::new(
			Equation::Add,
			BlendFactor::Value(BlendValue::SourceAlpha),
			BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
		)),
		..Default::default()
	};

	let material = load_material(
		&vertex_shader,
		&fragment_shader,
		MaterialParams {
			pipeline_params,
			uniforms: vec![
				("lowest_light_level".to_string(), UniformType::Float1),
				("window_height".to_string(), UniformType::Float1),
			],
			..Default::default()
		},
	)
	.unwrap();

	let label_style = root_ui().style_builder().text_color(WHITE).build();
	let skin = Skin {
		label_style,
		..root_ui().default_skin()
	};

	root_ui().push_skin(&skin);

	let mut visible_objects: Vec<Vec<Object>> = Vec::new();

	loop {
		while let Some(gilrs::Event {
			id,
			event: _,
			time: _,
		}) = gilrs.next_event()
		{
			active_gamepad = Some(id);
		}

		viewport_screen_height = screen_height() * (1.0 / NUM_PLAYERS as f32);
		frames_till_update_framerate -= 1;

		// If running at more than 60 fps, slow down
		// Logic
		players
			.iter_mut()
			.for_each(|player| player.update_enchantments());

		movement_input(
			&mut players[0],
			Some(0),
			&mut attacks,
			&TEXTURES,
			map.current_floor_mut(),
			&cameras[0],
		);

		if let Some(gamepad_id) = active_gamepad {
			let gamepad = gilrs.gamepad(gamepad_id);

			movement_input_controller(
				&mut players[1],
				Some(1),
				&mut attacks,
				&TEXTURES,
				map.current_floor_mut(),
				&gamepad,
			);

			door_interaction_input_controller(
				&players[1],
				&players,
				map.current_floor_mut(),
				&TEXTURES,
				&gamepad,
			)
		}

		door_interaction_input(&players[0], &players, map.current_floor_mut(), &TEXTURES);

		trigger_traps(&mut players, map.current_floor_mut(), &TEXTURES);
		set_effects(&mut players, map.current_floor_mut(), &TEXTURES);
		update_effects(&mut map.current_floor_mut().floor);
		update_cooldowns(&mut players);
		update_attacks(&mut players, map.current_floor_mut(), &mut attacks);
		update_monsters(
			&mut players,
			map.current_floor_mut(),
			&mut attacks,
			&TEXTURES,
		);

		if map.current_floor().should_descend(&players) {
			map.descend(&mut players);
		}

		// Rendering
		clear_background(BLACK);

		material.set_uniform("window_height", cameras[0].viewport.unwrap().3 as f32);

		visible_objects.clear();

		players.iter().for_each(|player| {
			visible_objects.push(
				Floor::visible_objects_mut(
					player,
					None,
					map.current_floor_mut().floor.objects_mut(),
				)
				.into_iter()
				.cloned()
				.collect(),
			)
		});

		let only_show_past_seen_objects = |obj: &&Object| -> bool {
			let is_currently_visible = visible_objects
				.iter()
				.flatten()
				.any(|v_obj| v_obj.tile_pos() == obj.tile_pos());

			obj.has_been_seen() && !is_currently_visible
		};

		// Draw all objects that have been seen
		let seen_objects = map
			.current_floor()
			.floor
			.objects()
			.iter()
			.filter(only_show_past_seen_objects);

		let monsters_to_draw = map.current_floor().monsters.iter().filter(|m| {
			let monster_tile_pos = pos_to_tile(&m.as_aabb());
			visible_objects
				.iter()
				.flatten()
				.any(|obj| obj.tile_pos() == monster_tile_pos)
		});

		cameras
			.iter_mut()
			.zip(players.iter())
			.enumerate()
			.for_each(|(i, (camera, player))| {
				camera.target = player.center();

				camera.zoom = Vec2::new(
					CAMERA_ZOOM,
					-CAMERA_ZOOM * (screen_width() / viewport_screen_height),
				) * 0.7;
				camera.viewport = Some((
					0,
					viewport_screen_height as i32 * i as i32,
					screen_width() as i32,
					viewport_screen_height as i32,
				));

				set_camera(camera);

				if player
					.enchantments()
					.get(&EnchantmentKind::Blinded)
					.is_none()
				{
					gl_use_material(material);
					material.set_uniform("lowest_light_level", 0.6_f32);

					visible_objects.iter().flatten().for_each(|o| {
						o.draw();
						o.items().iter().rev().for_each(|item| {
							item.draw();
						});
					});

					// Draw all monsters on top of a visible object tile
					monsters_to_draw.clone().for_each(|m| m.draw());

					material.set_uniform("lowest_light_level", 0.25_f32);

					seen_objects.clone().for_each(|o| {
						o.draw();
					});

					map.current_floor().exit().draw();

					material.set_uniform("lowest_light_level", 0.6_f32);
					visible_objects
						.iter()
						.flatten()
						.flat_map(|o| o.items().iter())
						.for_each(|i| i.draw());

					material.set_uniform("lowest_light_level", 1.0_f32);

					attacks.iter().for_each(|a| a.draw());
				}

				gl_use_default_material();
				players.iter().for_each(|p| p.draw());

				// Draw UI
				set_default_camera();
				draw_inventory(player);

				root_ui().label(
					Vec2::new(
						(camera.viewport.unwrap().2 - 150) as f32,
						camera.viewport.unwrap().1 as f32,
					),
					&format!("HP: {}", player.hp()),
				);
				root_ui().label(
					Vec2::new(
						(camera.viewport.unwrap().2 - 150) as f32,
						(camera.viewport.unwrap().1 + 10) as f32,
					),
					&format!("MP: {}", player.mp()),
				);

				if let Some(spell) = player.spells().first() {
					root_ui().label(
						Vec2::new(
							(camera.viewport.unwrap().2 - 150) as f32,
							(camera.viewport.unwrap().1 + 20) as f32,
						),
						&match player.changing_spell {
							false => format!("Spell: {}", spell),
							true => "Cycling Spell...".to_string(),
						},
					);
				}
			});

		root_ui().label(Vec2::ZERO, &fps.to_string());
		if SHOW_FRAMERATE && frames_till_update_framerate == 0 {
			let frame_time = get_frame_time();

			fps = (1.0 / frame_time).round();
			frames_till_update_framerate = 30;
		}
		next_frame().await
	}
}

fn window_conf() -> Conf {
	Conf {
		window_title: "Roguelite".to_string(),
		platform: Platform {
			//linux_backend: LinuxBackend::WaylandWithX11Fallback,
			..Default::default()
		},

		..Default::default()
	}
}
