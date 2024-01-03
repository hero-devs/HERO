use std::sync::Arc;

#[cfg(feature = "core")]
use uuid::Uuid;

use crate::IconData;
use crate::game::Game;
use crate::instance::Instance;

#[cfg(feature = "core")]
use crate::state::State;

pub struct Container {
	pub id: &'static str,
	pub icon: IconData,
	pub kind: ContainerKind,
	pub render: Box<dyn Fn() -> Vec<Element> + Send + Sync>
}

pub enum ContainerKind {
	InstancePage(Option<Box<dyn Fn(&Instance, Arc<Box<dyn Game>>) -> bool + Send + Sync>>)
}

pub enum Element {
	Text(Value<String>),
	PathSelect {
		kind: PathSelectKind,
		value: Value<String>
	}
}

pub enum PathSelectKind {
	File,
	Directory
}

#[derive(Clone)]
pub enum Value<T: From<String> + Clone> {
	Static(T),
	Link(ValueLink)
}

#[cfg(feature = "core")]
impl<T: From<String> + Clone> Value<T> {
	pub fn read(&self, instance: Option<&Instance>) -> Option<T> {
		match self.clone() {
			Value::Static(value) => Some(value),
			Value::Link(link) => match link {
				ValueLink::InstanceGameMeta(key) => instance.and_then(|x| x.game_meta.get::<T>(key.clone()))
			}
		}
	}

	pub fn write(self, value: impl Into<String>, instance_id: Option<Uuid>) {
		match self {
			Value::Link(link) => match link {
				ValueLink::InstanceGameMeta(key) => {
					let value: String = value.into();
					std::thread::spawn(move || {
						let state = State::get();
						let instances = &mut state.instances.write().items;
						if let Some(instance) = instance_id.and_then(|x| instances.get_mut(&x)) {
							instance.game_meta.set(key, value);
						}
					});
				}
			},
			_ => {}
		}
	}
}

impl<T: From<String> + Clone> From<T> for Value<T> {
	fn from(value: T) -> Self {
        Self::Static(value)
    }
}

#[derive(Clone)]
pub enum ValueLink {
	InstanceGameMeta(String)
}