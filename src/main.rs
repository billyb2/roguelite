mod attacks;
mod draw;
mod enchantments;
mod input;
mod map;
mod math;
mod monsters;
mod player;

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use attacks::*;
use draw::*;
use input::*;
use map::*;
use monsters::*;
use player::*;

use macroquad::miniquad::conf::Platform;
use macroquad::prelude::*;
use macroquad::ui::root_ui;

const DEFAULT_FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
uniform lowp vec2 player_pos;

void main() {
    gl_FragColor = texture2D(Texture, uv);
	vec2 frag_coord = gl_FragCoord.xy;
	frag_coord.y = 600.0 - gl_FragCoord.y;

	float lighting = 1.0 - min(length(frag_coord - player_pos), 300.0) / 300.0;
	gl_FragColor.rgb *= vec3(lighting * 0.8);
}
";

const DEFAULT_VERTEX_SHADER: &'static str = "#version 100
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

#[macroquad::main(window_conf)]
async fn main() {
	let time = SystemTime::elapsed(&UNIX_EPOCH).unwrap().as_secs();
	rand::srand(time);
	print!("What player class are you? Warrior or Wizard?: ");
	io::stdout().flush().unwrap();

	let mut class = String::new();
	// io::stdin().read_line(&mut class).unwrap();

	class = class.to_lowercase();
	class.pop();

	let class = match class.is_empty() {
		true => PlayerClass::Wizard,
		false => match class.as_str() {
			"warrior" => PlayerClass::Warrior,
			"wizard" | "." => PlayerClass::Wizard,
			c => panic!("Invalid class given: {c}"),
		},
	};

	let textures: HashMap<String, Texture2D> = fs::read_dir("assets")
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
		.collect();

	let mut monsters: Vec<Box<dyn Monster>> = Vec::new();

	let mut map = Map::new(&textures, &mut monsters);
	let mut players = vec![Player::new(class, map.current_floor().current_spawn())];

	let mut camera = Camera2D {
		target: players[0].pos(),
		zoom: Vec2::new(
			CAMERA_ZOOM,
			-CAMERA_ZOOM * (screen_width() / screen_height()),
		),
		..Default::default()
	};

	const SHOW_FRAMERATE: bool = true;

	let mut frames_till_update_framerate: u8 = 30;
	let mut fps = 0.0;

	let fragment_shader = DEFAULT_FRAGMENT_SHADER.to_string();
	let vertex_shader = DEFAULT_VERTEX_SHADER.to_string();

	let pipeline_params = PipelineParams {
		depth_write: true,
		depth_test: Comparison::LessOrEqual,
		..Default::default()
	};

	let material = load_material(
		&vertex_shader,
		&fragment_shader,
		MaterialParams {
			pipeline_params,
			uniforms: vec![("player_pos".to_string(), UniformType::Float2)],
			..Default::default()
		},
	)
	.unwrap();

	loop {
		let frame_time = get_frame_time();
		frames_till_update_framerate -= 1;

		// If running at more than 60 fps, slow down
		if frame_time < 1.0 / 60.0 {
			let time_to_sleep = ((1.0 / 60.0) - frame_time) * 1000.0;
			std::thread::sleep(std::time::Duration::from_millis(time_to_sleep as u64));
		}

		// Logic
		keyboard_input(
			&mut players[0],
			&mut monsters,
			&textures,
			map.current_floor(),
		);
		update_cooldowns(&mut players);
		update_attacks(&mut players, &mut monsters, map.current_floor());
		update_monsters(&mut monsters, &mut players, map.current_floor());

		if map.current_floor().should_descend(&players, &monsters) {
			map.descend(&mut players, &mut monsters, &textures)
		}

		// Rendering
		clear_background(WHITE);

		camera.target = players[0].pos();
		camera.zoom.y = -0.005 * (screen_width() / screen_height());
		set_camera(&camera);

		material.set_uniform("player_pos", camera.world_to_screen(players[0].pos));

		gl_use_material(material);

		map.draw();
		monsters.iter().for_each(|m| m.draw());

		players
			.iter()
			.flat_map(|p| p.attacks.iter())
			.for_each(|a| a.draw());

		gl_use_default_material();

		players.iter().for_each(|p| p.draw());

		root_ui().label(Vec2::ZERO, &fps.to_string());

		if SHOW_FRAMERATE && frames_till_update_framerate == 0 {
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
