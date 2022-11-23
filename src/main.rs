mod attacks;
mod draw;
mod enchantments;
mod input;
mod items;
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
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use map::*;
use monsters::*;
use player::*;

use macroquad::miniquad::conf::Platform;
use macroquad::prelude::*;
use macroquad::ui::{root_ui, Skin};

use rayon::prelude::*;

pub const MAX_VIEW_OF_PLAYER: f32 = 200.0;

const DEFAULT_FRAGMENT_SHADER: &str = "
#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
uniform lowp vec2 player_pos;
uniform lowp float lowest_light_level;
uniform lowp float window_height;
const lowp float VISION_SIZE = 400.0;

void main() {
    gl_FragColor = texture2D(Texture, uv);

	float lighting = 1.0 - min(length(gl_FragCoord.xy - player_pos), VISION_SIZE) / VISION_SIZE;
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

#[macroquad::main(window_conf)]
async fn main() {
	let time = SystemTime::elapsed(&UNIX_EPOCH).unwrap().as_secs();
	rand::srand(time);
	print!("What player class are you? Warrior or Wizard?: ");
	io::stdout().flush().unwrap();

	let mut class = String::new();
	io::stdin().read_line(&mut class).unwrap();

	class = class.to_lowercase();
	class.pop();

	let class = match class.is_empty() {
		true => PlayerClass::Warrior,
		false => match class.as_str().try_into() {
			Ok(class) => class,
			Err(_) => panic!("Invalid class given: {class}"),
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
	let mut attacks = Vec::new();

	let mut map = Map::new(&textures, &mut monsters);
	let mut players = vec![Player::new(
		0,
		class,
		map.current_floor().current_spawn(),
		&textures,
	)];

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
				("player_pos".to_string(), UniformType::Float2),
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

	loop {
		let frame_time = get_frame_time();
		frames_till_update_framerate -= 1;

		// If running at more than 60 fps, slow down
		if frame_time < 1.0 / 60.0 {
			let time_to_sleep = ((1.0 / 60.0) - frame_time) * 1000.0;
			std::thread::sleep(std::time::Duration::from_millis(time_to_sleep as u64));
		}

		// Logic
		movement_input(
			&mut players[0],
			&mut attacks,
			&textures,
			map.current_floor_mut(),
		);

		door_interaction_input(
			&players[0],
			&players,
			&monsters,
			map.current_floor_mut(),
			&textures,
		);

		trigger_traps(&mut players, map.current_floor_mut());
		update_cooldowns(&mut players);
		update_attacks(
			&mut monsters,
			&mut players,
			map.current_floor(),
			&mut attacks,
		);
		update_monsters(&mut monsters, &mut players, map.current_floor());

		if map.current_floor().should_descend(&players, &monsters) {
			map.descend(&mut players, &mut monsters, &textures)
		}

		// Rendering
		clear_background(BLACK);

		camera.target = players[0].pos();
		material.set_uniform("window_height", screen_height());
		camera.zoom.y = -CAMERA_ZOOM * (screen_width() / screen_height());
		set_camera(&camera);

		material.set_uniform("player_pos", camera.world_to_screen(players[0].pos));

		gl_use_material(material);

		// Draw all visible objects
		let visible_objects =
			Floor::visible_objects_mut(&players[0], None, &mut map.current_floor_mut().objects);

		material.set_uniform("lowest_light_level", 1.0_f32);

		visible_objects.iter().for_each(|o| {
			o.draw();
		});

		// Draw all monsters on top of a visible object tile
		monsters
			.iter()
			.filter(|m| {
				let monster_tile_pos = pos_to_tile(&m.as_aabb());
				let should_be_drawn = visible_objects
					.iter()
					.any(|obj| obj.tile_pos() == monster_tile_pos);

				should_be_drawn
			})
			.for_each(|m| m.draw());

		material.set_uniform("lowest_light_level", 0.65_f32);

		let visible_objects = map.current_floor().visible_objects(&players[0], None);

		visible_objects
			.iter()
			.flat_map(|o| o.items().iter())
			.for_each(|i| {
				i.draw();
			});

		let only_show_past_seen_objects = |obj: &&Object| -> bool {
			let is_currently_visible = visible_objects
				.iter()
				.any(|v_obj| v_obj.tile_pos() == obj.tile_pos());

			obj.has_been_seen() && !is_currently_visible
		};

		// Draw all objects that have been seen
		map.current_floor()
			.objects
			.par_iter()
			.filter(only_show_past_seen_objects)
			.collect::<Vec<&Object>>()
			.into_iter()
			.for_each(|o| {
				o.draw();
			});

		map.current_floor().exit().draw();

		material.set_uniform("lowest_light_level", 1.0_f32);

		attacks.iter().for_each(|a| a.draw());

		gl_use_default_material();

		players.iter().for_each(|p| p.draw());

		root_ui().label(Vec2::ZERO, &fps.to_string());
		root_ui().label(
			Vec2::new(screen_width() - 100.0, 0.0),
			&format!("HP: {}", players[0].hp()),
		);
		root_ui().label(
			Vec2::new(screen_width() - 100.0, 10.0),
			&format!("MP: {}", players[0].mp()),
		);

		if !players[0].spells().is_empty() {
			root_ui().label(
				Vec2::new(screen_width() - 150.0, 20.0),
				&match players[0].changing_spell {
					false => format!("Spell: {}", players[0].spells()[0]),
					true => format!("Cycling Spell..."),
				},
			);
		}

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
