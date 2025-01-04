pub struct Downloader {
	items: Vec<Download>
}

impl Downloader {
	pub fn new() -> Self {
		Self {
			items: vec![]
		}
	}

	pub fn download_files_over_http(urls: Vec<String>) -> DownloadGroup {

	}
}

pub struct Download {

}

impl Download {
	pub fn stop(&self) {
		
	}
}

pub struct DownloadGroup {
	items: Vec<&Download>
}

impl DownloadGroup {
	pub fn stop_all(&self) {
		for download in self.items {
			download.stop();
		}
	}
}