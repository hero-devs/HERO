pub mod game;
pub mod state;
pub mod content;
pub mod storage;
pub mod instance;
pub mod extension;
pub mod downloader;
pub mod localisation;

#[cfg(feature = "ui")]
pub mod user_interface;

pub use uuid;
pub use parking_lot;

pub trait Icon {
	fn icon(&self) -> Option<IconData>;
}

#[derive(Clone)]
pub struct IconData {
	pub path: String,
	pub bytes: Vec<u8>
}

#[cfg(feature = "egui")]
impl From<IconData> for egui::ImageSource<'static> {
    fn from(value: IconData) -> Self {
        Self::Bytes { uri: value.path.into(), bytes: value.bytes.into() }
    }
}

#[macro_export]
macro_rules! impl_icon {
	($t: ident, $path: literal) => {
		impl $crate::Icon for $t {
			fn icon(&self) -> Option<$crate::IconData> {
				Some($crate::include_icon!($path))
			}
		}
	};
}

#[macro_export]
macro_rules! include_icon {
	($path: literal) => {
		$crate::IconData {
			path: $path.to_string(),
			bytes: include_bytes!($path).to_vec()
		}
	};
}