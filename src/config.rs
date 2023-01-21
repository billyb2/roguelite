use std::time::{Duration, Instant};

use crate::init_game::{init_players, GameInfo};
use crate::net::{init_net, GGRSConfig};
use crate::player::PlayerClass;
use crate::NET_SESSION;

#[derive(Clone)]
pub struct ConfigInfo {
	player_config_info: PlayerConfigInfo,
	net_config_info: GGRSConfig,
}

impl Default for ConfigInfo {
	fn default() -> Self {
		Self {
			player_config_info: PlayerConfigInfo::default(),
			net_config_info: GGRSConfig::default(),
		}
	}
}

impl ConfigInfo {
	pub fn set_class(&mut self, class: PlayerClass) { self.player_config_info.class = class; }

	pub fn class(&self) -> PlayerClass { self.player_config_info.class }

	pub fn local_port(&self) -> u16 { self.net_config_info.local_port }

	pub fn set_local_port(&mut self, local_port: u16) {
		self.net_config_info.local_port = local_port;
	}

	pub fn remote_port(&self) -> u16 { self.net_config_info.remote_port }

	pub fn set_remote_port(&mut self, remote_port: u16) {
		self.net_config_info.remote_port = remote_port;
	}

	pub fn set_config(&self, game_info: &mut GameInfo) {
		game_info.accumulator = Duration::ZERO;
		game_info.last_update = Instant::now();

		game_info.game_state.players =
			init_players(self.player_config_info.class, &game_info.game_state.map);
		unsafe { NET_SESSION = Some(init_net(&game_info.config_info.net_config_info)) };
	}
}

#[derive(Clone)]
pub struct PlayerConfigInfo {
	pub class: PlayerClass,
}

impl Default for PlayerConfigInfo {
	fn default() -> Self {
		Self {
			class: PlayerClass::Warrior,
		}
	}
}
