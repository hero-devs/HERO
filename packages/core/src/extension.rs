use std::sync::Arc;

#[cfg(feature = "core")]
use std::path::PathBuf;
use crate::Icon;
use crate::game::Game;
use crate::downloader::Downloader;

#[cfg(feature = "core")]
use crate::state::State;

#[cfg(feature = "ui")]
use crate::user_interface::Container;

use crate::content::ContentProvider;

pub trait Extension: Icon + Sync + Send {
	/// The unique identifier for your extension.
	fn id(&self) -> &'static str;

	/// Runs after the extension has been fully loaded & registered.
	fn setup(&self) {}

	fn games(&self) -> Option<Vec<Arc<Box<dyn Game>>>> {
		None
	}

	fn downloader(&self) -> Option<Arc<Downloader>> {
		None
	}

	fn localisation(&self) -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
		vec![]
	}

	#[cfg(feature = "ui")]
	fn ui_containers(&self) -> Vec<Container> {
		vec![]
	}

	fn content_providers(&self) -> Vec<Box<dyn ContentProvider>> {
		vec![]
	}
}

pub struct Extensions {
	pub items: Vec<Box<dyn Extension>>,

	#[cfg(feature = "core")]
	libraries: Vec<Arc<libloading::Library>>
}

#[cfg(feature = "core")]
impl Extensions {
	pub fn new() -> Self {
		Self {
			items: Vec::new(),
			libraries: Vec::new()
		}
	}

	pub fn load_libraries(&mut self, path: PathBuf) {
		if let Ok(entries) = std::fs::read_dir(path) {
			for entry in entries.filter_map(|x| x.ok()) {
				let n = entry.file_name();
				let name = n.to_string_lossy();
				if name.ends_with(".dll") || name.ends_with(".dylib") {
					match unsafe { libloading::Library::new(entry.path()) } {
						Ok(lib) => {
							self.libraries.push(Arc::new(lib));
							
							/*let lib = self.libraries.last().unwrap();
							let new_ext: libloading::Symbol<extern "Rust" fn(&mut crate::state::State) -> Box<dyn Extension>> = unsafe { lib.get(b"build_hero_extension") }
								.expect("oh no!");
							let extension = new_ext(state);
							extension.setup();

							self.items.push(extension);*/
						}
						Err(err) => {
							println!("failed to load extension! {}", err);
						}
					}
				}
			}
		}
	}
}

#[cfg(feature = "core")]
impl Drop for Extensions {
    fn drop(&mut self) {
		println!("dropping extensions into the void");
        if !self.items.is_empty() || !self.libraries.is_empty() {
			for _extension in self.items.drain(..) {
				
			}
            for library in self.libraries.drain(..) {
				drop(library);
			}
        }
    }
}

#[cfg(feature = "core")]
pub fn load_extensions() {
	let state = State::get();
	let mut extensions = state.extensions.write();

	let libraries = &extensions.libraries;
	let mut localisation = state.localisation.write();

	#[cfg(feature = "ui")]
	let mut ui_containers = state.ui_containers.write();
	for library in libraries.clone() {
		let new_ext: libloading::Symbol<extern "Rust" fn(&crate::state::State) -> Box<dyn Extension>> = unsafe { library.get(b"build_hero_extension") }
			.expect("oh no!");
		let extension = new_ext(&state);
		extension.setup();

		for (locale, data) in extension.localisation() {
			localisation.insert_data(locale, data);
		}

		#[cfg(feature = "ui")]
		for container in extension.ui_containers() {
			ui_containers.push(Arc::new(container));
		}

		extensions.items.push(extension);
	}
}