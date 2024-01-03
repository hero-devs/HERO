// based off https://github.com/FabianLars/tauri-plugin-deep-link, unfinished
use std::io::{ Error, Result, ErrorKind };
use std::path::{ Path, PathBuf };
use ctor::ctor;
use once_cell::sync::OnceCell;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod platform_impl;

// TODO
#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform_impl;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod platform_impl;

static ID: OnceCell<String> = OnceCell::new();

pub fn set_identifier(identifier: &str) -> Result<()> {
    ID.set(identifier.to_string())
        .map_err(|_| ErrorKind::AlreadyExists.into())
}

pub fn register<F: FnMut(String) + Send + 'static>(scheme: &str, handler: F) -> Result<()> {
    platform_impl::register(scheme, handler)
}

pub fn listen<F: FnMut(String) + Send + 'static>(handler: F) -> Result<()> {
    platform_impl::listen(handler)
}

pub fn unregister(scheme: &str) -> Result<()> {
    platform_impl::unregister(scheme)
}

pub fn prepare(identifier: &str) {
    platform_impl::prepare(identifier)
}

#[ctor]
static STARTING_BINARY: StartingBinary = StartingBinary::new();

struct StartingBinary(Result<PathBuf>);

impl StartingBinary {
	fn new() -> Self {
		let dangerous_path = match std::env::current_exe() {
			Ok(dangerous_path) => dangerous_path,
			error @ Err(_) => return Self(error),
		};

		if let Some(symlink) = Self::has_symlink(&dangerous_path) {
			return Self(Err(Error::new(
				ErrorKind::InvalidData,
				format!("StartingBinary found current_exe() that contains a symlink on a non-allowed platform: {}", symlink.display()),
			)));
		}

		Self(dangerous_path.canonicalize())
	}

	fn cloned(&self) -> Result<PathBuf> {
		self
			.0
			.as_ref()
			.map(Clone::clone)
			.map_err(|e| Error::new(e.kind(), e.to_string()))
	}

	#[cfg(not(target_os = "macos"))]
	fn has_symlink(_: &Path) -> Option<&Path> {
		None
	}
	
	#[cfg(target_os = "macos")]
	fn has_symlink(path: &Path) -> Option<&Path> {
		path.ancestors().find(|ancestor| {
			matches!(
				ancestor
					.symlink_metadata()
					.as_ref()
					.map(std::fs::Metadata::file_type)
					.as_ref()
					.map(std::fs::FileType::is_symlink),
				Ok(true)
			)
		})
	}
}