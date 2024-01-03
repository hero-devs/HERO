use std::path::PathBuf;
use std::collections::HashMap;
use uuid::Uuid;
use serde::Deserialize;

use crate::{
	storage::{ PLUTO, read_pluto_proto_file },
	Icon,
	IconData
};
#[cfg(feature = "core")]
use crate::state::State;

pub struct Instance {
	pub path: PathBuf,
	pub metadata: InstanceMetadata,
	pub game_meta: PLUTO
}

#[derive(Deserialize)]
pub struct InstanceMetadata {
	pub id: Uuid,
	pub author: String,
	pub icon_path: Option<std::path::PathBuf>,
	pub created_at: String,
	pub display_names: DisplayNames
}

#[derive(Deserialize)]
pub struct DisplayNames {
	pub custom: String
}

impl Instance {
	#[cfg(feature = "core")]
	pub fn new(path: PathBuf, game_id: impl ToString) -> Option<Self> {
		if let Ok(data) = std::fs::read_to_string(&path.join("instance_meta.json")) {
			let metadata: InstanceMetadata = serde_json::from_str(&data).unwrap();
			println!("loaded instance {}", metadata.id);
			return Some(Self {
				path: path.clone(),
				metadata,
				game_meta: read_pluto_proto_file(path.join(format!("{}_meta.PLUTO_PROTO", game_id.to_string()))).unwrap()
			});
		}
		println!("instance_meta.json not found in {:?}", path);
		None
	}

	pub fn id(&self) -> Uuid {
		self.metadata.id.clone()
	}

	pub fn display_name(&self) -> String {
		self.metadata.display_names.custom.clone()
	}
}

impl Icon for Instance {
	fn icon(&self) -> Option<IconData> {
		let path = self.path.join(self.metadata.icon_path.clone().unwrap());
		if let Ok(bytes) = std::fs::read(&path) {
			return Some(IconData {
				path: path.to_string_lossy().to_string(),
				bytes
			});
		}
		None
	}
}

pub struct Instances {
	pub items: HashMap<Uuid, Instance>
}

impl Instances {
	pub fn new() -> Self {
		Self {
			items: HashMap::new()
		}
	}
}

#[cfg(feature = "core")]
pub fn load_instance(_path: PathBuf) {

}

#[cfg(feature = "core")]
pub fn load_instances() {
	let state = State::get();
	if let Some(game) = state.get_current_game() {
		let path = state.path.join("instances").join(game.id());
		println!("loading instances from {:?}", path);

		let instances = &mut state.instances.write().items;
		if let Ok(entries) = std::fs::read_dir(path) {
			for entry in entries.filter_map(|x| x.ok()) {
				if entry.file_type().unwrap().is_dir() {
					if let Some(instance) = Instance::new(entry.path(), game.id()) {
						instances.insert(instance.metadata.id.clone(), instance);
					}
				}
			}
		}
	}
}

#[cfg(feature = "core")]
pub async fn launch(instance_id: Uuid) {
	let state = State::get();
	if let Some(game) = state.get_current_game() {
		if let Some(instance) = state.instances.read().items.get(&instance_id) {
			println!("launching instance {}", instance_id);
			game.launch(&instance);
		}
	}
}