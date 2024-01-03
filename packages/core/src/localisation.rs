use std::collections::HashMap;

pub struct Localisation {
	data: HashMap<String, HashMap<String, String>>,
	current_locale: String
}

impl Localisation {
	pub fn new() -> Self {
		Self {
			data: HashMap::new(),
			current_locale: "en-AU".into()
		}
	}

	pub fn translate(&self, key: String) -> String {
		if let Some(data) = self.data.get(&self.current_locale) {
			if let Some(value) = data.get(&key) {
				return value.clone();
			}
		}
		key
	}

	#[cfg(feature = "core")]
	pub fn insert_data(&mut self, locale: &'static str, data: Vec<(&'static str, &'static str)>) {
		let name = locale.to_string();
		if let Some(current) = self.data.get_mut(&name) {
			for (key, value) in data {
				current.insert(key.to_string(), value.to_string());
			}
		} else {
			self.data.insert(name, data.into_iter().map(|x| (x.0.to_string(), x.1.to_string())).collect());
		}
	}
}