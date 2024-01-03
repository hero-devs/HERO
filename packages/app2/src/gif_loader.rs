use std::{ mem::size_of, path::Path, sync::Arc, io::Cursor };
use std::time::{ UNIX_EPOCH, SystemTime };
use image::codecs::gif::GifDecoder;
use image::AnimationDecoder;
use eframe::egui::{
	self,
    ahash::HashMap,
    load::{BytesPoll, ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizeHint},
    mutex::Mutex,
    ColorImage,
};

type Entry = Result<Arc<Gif>, String>;

struct Gif {
	length: u128,
	frames: Vec<(Arc<ColorImage>, u32)>,
	created_at: SystemTime
}

#[derive(Default)]
pub struct GifLoader {
    cache: Mutex<HashMap<String, Entry>>
}

impl GifLoader {
    pub const ID: &str = egui::generate_loader_id!(GifLoader);
}

fn is_supported_uri(uri: &str) -> bool {
    let Some(ext) = Path::new(uri).extension().and_then(|ext| ext.to_str()) else {
        // `true` because if there's no extension, assume that we support it
        return true;
    };

    ext == "gif"
}

fn is_unsupported_mime(mime: &str) -> bool {
    !mime.contains("gif")
}

impl ImageLoader for GifLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &egui::Context, uri: &str, _: SizeHint) -> ImageLoadResult {
        // three stages of guessing if we support loading the image:
        // 1. URI extension
        // 2. Mime from `BytesPoll::Ready`
        // 3. image::guess_format

        // (1)
		println!("{}", uri);
        if !is_supported_uri(uri) {
            return Err(LoadError::NotSupported);
        }

        let mut cache = self.cache.lock();
        if let Some(entry) = cache.get(uri).cloned() {
            match entry {
                Ok(image) => {
					let now = SystemTime::now();
					let time = now.duration_since(UNIX_EPOCH).unwrap().as_millis() % image.length;

					let mut t = 0 as u128;
					let frame = image.frames.iter().find(|x| {
						t += x.1 as u128;
						time <= t
					}).unwrap();
					Ok(ImagePoll::Ready { image: frame.0.clone() })
				},
                Err(err) => Err(LoadError::Loading(err)),
            }
        } else {
            match ctx.try_load_bytes(uri) {
                Ok(BytesPoll::Ready { bytes, mime, .. }) => {
                    // (2 and 3)
                    if mime.as_deref().is_some_and(is_unsupported_mime)
                        || image::guess_format(&bytes).is_err()
                    {
                        return Err(LoadError::NotSupported);
                    }

                    println!("started loading {uri:?}");
					let decoder = GifDecoder::new(Cursor::new(bytes)).unwrap();

					let frames = decoder
						.into_frames()
						.collect_frames()
						.unwrap();

					let frames: Vec<(Arc<ColorImage>, u32)> = frames
                		.iter()
						.enumerate()
						.map(|(_i, f)| {
							let image = Arc::new(ColorImage::from_rgba_unmultiplied(
								[f.buffer().width() as _, f.buffer().height() as _],
								f.buffer(),
							));
							let (num, den) = f.delay().numer_denom_ms();
							(image, (num as f32 * 1000.0 / den as f32).round() as u32)
						})
						.collect();

					let gif = Arc::new(Gif {
						frames: frames.clone(),
						length: frames.iter().map(|x| x.1 as u128).sum(),
						created_at: SystemTime::now()
					});

                    println!("finished loading {uri:?}");
                    cache.insert(uri.into(), Ok(gif.clone()));
                    Ok(ImagePoll::Ready { image: gif.frames[0].0.clone() })
                }
                Ok(BytesPoll::Pending { size }) => Ok(ImagePoll::Pending { size }),
                Err(err) => Err(err),
            }
        }
    }

    fn forget(&self, uri: &str) {
        let _ = self.cache.lock().remove(uri);
    }

    fn forget_all(&self) {
        self.cache.lock().clear();
    }

    fn byte_size(&self) -> usize {
        self.cache
            .lock()
            .values()
            .map(|result| match result {
                Ok(image) => image.frames.iter().map(|x| x.0.pixels.len() * size_of::<egui::Color32>()).sum(),
                Err(err) => err.len(),
            })
            .sum()
    }
}