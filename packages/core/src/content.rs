use std::any::Any;
use crate::instance::Instance;

#[derive(Clone, Debug)]
pub struct ContentFile {
	pub name: String,
	pub icon: Option<Vec<u8>>,
	pub version: Option<String>
}

#[derive(Clone, Debug)]
pub struct ContentProviderItem {
	/// Unique identifier.
	pub id: String,

	/// Name to display on the frontend.
	pub name: String,

	/// Icon to display on the frontend, must be a URL.
	pub icon: Option<String>,

	/// Author of the published item.
	pub authors: Option<Vec<ContentProviderItemAuthor>>,

	/// Short summary to display on the frontend.
	pub summary: Option<String>,

	/// Whether or not this item can be installed.
	pub can_be_installed: bool
}

#[derive(Clone, Debug)]
pub struct ContentProviderItemAuthor {
	pub id: String,
	pub name: String,
	pub website_url: Option<String>
}

#[derive(Debug)]
pub enum ContentProviderSearchSortType {
	Relevance,
	DownloadCount,
	Rating,
	RecentlyPublished,
	RecentlyUpdated
}

pub trait ContentProvider: Any + Send + Sync {
	/// The unique identifier for this content provider.
	fn name(&self) -> &'static str;
	fn search(&mut self, query: String, sort_type: ContentProviderSearchSortType, descending: bool) -> Vec<ContentProviderItem>;
	fn install(&mut self, instance: &Instance, item_id: &String);

	/// Defines if and how long search queries should be cached for.
	fn cache_duration(&self) -> Option<std::time::Duration> {
		None
	}

	/// An array containing the names of games this content provider supports.
	fn supported_types(&self) -> Vec<&'static str> {
		vec![]
	}
}

pub trait ContentPage: Any + Send + Sync {
	fn name(&self) -> &'static str;
	fn items(&self, _instance: &Instance) -> Vec<ContentFile> {
		Vec::new()
	}
}