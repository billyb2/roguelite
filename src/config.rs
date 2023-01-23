use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};
use std::{fs, io};

use ron::error::SpannedError;
use serde::{Deserialize, Serialize};

use crate::init_game::{init_players, GameInfo};
use crate::net::{init_net, GGRSConfig};
use crate::player::PlayerClass;
use crate::NET_SESSION;

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Debug)]
pub enum ConfigError {
	#[cfg(feature = "native")]
	Io(io::Error),
	DeRonErr(SpannedError),
	SeRonErr(ron::Error),
}

impl From<io::Error> for ConfigError {
	fn from(value: io::Error) -> Self { ConfigError::Io(value) }
}

impl From<SpannedError> for ConfigError {
	fn from(value: SpannedError) -> Self { ConfigError::DeRonErr(value) }
}

impl From<ron::Error> for ConfigError {
	fn from(value: ron::Error) -> Self { ConfigError::SeRonErr(value) }
}

impl ConfigInfo {
	#[cfg(feature = "native")]
	pub fn new(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
		let config: String = fs::read_to_string(path)?;
		Ok(ron::from_str(&config)?)
	}

	pub fn set_class(&mut self, class: PlayerClass) {
		self.player_config_info.class = class;
		self.save_to_disk().unwrap();
	}

	pub fn class(&self) -> PlayerClass { self.player_config_info.class }

	pub fn local_port(&self) -> u16 { self.net_config_info.local_port }

	pub fn multiplayer(&self) -> bool { self.net_config_info.multiplayer }

	pub fn set_opposite_multiplayer(&mut self) {
		self.net_config_info.multiplayer = !self.net_config_info.multiplayer;
		self.save_to_disk().unwrap();
	}

	pub fn set_local_port(&mut self, local_port: u16) {
		self.net_config_info.local_port = local_port;
		self.save_to_disk().unwrap();
	}

	pub fn remote_port(&self) -> u16 { self.net_config_info.remote_port }

	pub fn set_remote_port(&mut self, remote_port: u16) {
		self.net_config_info.remote_port = remote_port;
		self.save_to_disk().unwrap();
	}

	pub fn set_config(&self, game_info: &mut GameInfo) {
		game_info.accumulator = Duration::ZERO;
		game_info.last_update = Instant::now();

		let num_players = match self.multiplayer() {
			true => 2,
			false => 1,
		};

		game_info.game_state.players = init_players(
			self.player_config_info.class,
			&game_info.game_state.map,
			num_players,
		);
		unsafe { NET_SESSION = Some(init_net(&game_info.config_info.net_config_info)) };
	}

	#[cfg(feature = "native")]
	fn save_to_disk(&self) -> Result<(), ConfigError> {
		let mut file = fs::File::create(".game_config").unwrap();
		let serialized_config = ron::to_string(self)?;

		file.write_all(serialized_config.as_bytes())?;

		Ok(())
	}

	#[cfg(not(feature = "native"))]
	fn save_to_disk(&self) -> Result<(), ConfigError> { Ok(()) }
}

#[derive(Clone, Serialize, Deserialize)]
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
