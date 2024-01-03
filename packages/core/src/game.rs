use crate::Icon;
use crate::content::ContentPage;
use crate::instance::Instance;

pub trait Game: Icon + Sync + Send {
	/// The unique identifier for your game.
	fn id(&self) -> &'static str;

	fn launch(&self, instance: &Instance);

	fn pre_load(&self) {}

	fn content_pages(&self) -> Vec<Box<dyn ContentPage>> {
		Vec::new()
	}
}