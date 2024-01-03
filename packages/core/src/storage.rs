use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
pub struct PLUTO {
	path: PathBuf,
	items: HashMap<String, String>
}

impl PLUTO {
	pub fn new(path: PathBuf, items: HashMap<String, String>) -> Self {
		println!("{:?}", items);
		Self { path, items }
	}

	pub fn get<T: From<String>>(&self, key: impl Into<String>) -> Option<T> {
		self.items.get(&key.into()).map(|x| x.clone().into())
	}

	pub fn set<T: Into<String>>(&mut self, key: impl Into<String>, value: T) {
		self.items.insert(key.into(), value.into());

		let data = self.items.iter().enumerate().map(|(_,x)| format!("{}\n{}", x.0, x.1)).collect::<Vec<String>>().join("\n");
		fs::write(&self.path, data).unwrap();
	}
}

pub fn read_pluto_proto_file(path: PathBuf) -> Result<PLUTO, ()> {
	if path.exists() {
		if let Ok(data) = fs::read_to_string(&path) {
			let mut items = HashMap::new();
	
			let lines = data.lines();
			for a in lines.clone().step_by(2).zip(lines.skip(1).step_by(2)) {
				println!("{:?}", a);
				items.insert(a.0.into(), a.1.into());
			}
	
			return Ok(PLUTO::new(path, items));
		}
	} else {
		return Ok(PLUTO::new(path, HashMap::new()))
	}
	Err(())
}