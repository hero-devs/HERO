use std::sync::Arc;
use std::path::PathBuf;

#[cfg(feature = "core")]
use std::sync::OnceLock;

use std::collections::HashMap;
use uuid::Uuid;
use parking_lot::RwLock;

use crate::game::Game;
use crate::instance::Instances;
use crate::extension::Extensions;
use crate::localisation::Localisation;

#[cfg(feature = "ui")]
use crate::user_interface::Container;

#[cfg(feature = "core")]
static CORE_STATE: OnceLock<RwLock<State>> = OnceLock::new();

pub struct State {
	pub path: PathBuf,
	pub instances: RwLock<Instances>,
	pub extensions: RwLock<Extensions>,
	pub current_game: RwLock<Option<String>>,
	pub localisation: RwLock<Localisation>,
	pub loading_bars: RwLock<HashMap<Uuid, LoadingBar>>,

	#[cfg(feature = "ui")]
	pub ui_containers: RwLock<Vec<Arc<Container>>>
}

pub struct LoadingBar {
	pub total: f64,
    pub current: f64,
	pub bar_type: LoadingBarType
}

pub enum LoadingBarType {
	LoadGame {
		game_id: String
	}
}

impl State {
	#[cfg(feature = "core")]
	pub fn get() -> Arc<parking_lot::RwLockReadGuard<'static, Self>> {
		Arc::new(
			CORE_STATE.get_or_init(Self::initialise)
			.read()
		)
	}

	#[cfg(feature = "core")]
	pub fn initialise() -> RwLock<Self> {
		RwLock::new(Self {
			path: dirs::config_dir().unwrap().join("HAKUMI").join("HERO"),
			instances: RwLock::new(Instances::new()),
			extensions: RwLock::new(Extensions::new()),
			current_game: RwLock::new(None),
			localisation: RwLock::new(Localisation::new()),
			loading_bars: RwLock::new(HashMap::new()),

			#[cfg(feature = "ui")]
			ui_containers: RwLock::new(vec![])
		})
	}

	pub fn get_games(&self) -> Vec<Arc<Box<dyn Game>>> {
		self.extensions.read().items.iter().filter_map(|x| x.games()).flatten().collect()
	}

	pub fn get_game(&self, id: String) -> Option<Arc<Box<dyn Game>>> {
		for extension in self.extensions.read().items.iter() {
			if let Some(games) = extension.games() {
				for game in games {
					if game.id().to_string() == id {
						return Some(game);
					}
				}
			}
		}
		None
	}

	pub fn get_current_game(&self) -> Option<Arc<Box<dyn Game>>> {
		self.current_game.read().clone().and_then(|x| self.get_game(x))
	}

	#[cfg(feature = "ui")]
	pub fn get_ui_container(&self, id: impl Into<String>) -> Option<Arc<Container>> {
		let id: String = id.into();
		self.ui_containers.read().iter().cloned().find(|x| x.id == id)
	}

	#[cfg(feature = "core")]
	pub fn init_loading(&self, bar_type: LoadingBarType, total: f64) -> Uuid {
		let id = Uuid::new_v4();
		self.loading_bars.write().insert(id, LoadingBar {
			total,
			current: 0.,
			bar_type
		});

		id
	}

	#[cfg(feature = "core")]
	pub fn add_loading(&self, bar_id: &Uuid, amount: f64) {
		let mut bars = self.loading_bars.write();
		if let Some(bar) = bars.get_mut(&bar_id) {
			bar.current += amount;
			if bar.current >= bar.total {
				bars.remove(&bar_id);
			}
		}
	}

	pub fn t(&self, key: String) -> String {
		self.localisation.read().translate(key)
	}
}

#[cfg(feature = "core")]
pub async fn load_game(game_id: String) {
	let state = State::get();
	*state.current_game.write() = Some(game_id.clone());

	let id = state.init_loading(LoadingBarType::LoadGame { game_id: game_id.clone() }, 2.);

	let game = state.get_game(game_id).unwrap();
	game.pre_load();

	state.add_loading(&id, 1.);
	crate::instance::load_instances();

	state.add_loading(&id, 1.);
}