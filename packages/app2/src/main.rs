#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::collections::HashMap;
use image::EncodableLayout;
use eframe::{
	egui::{ self, pos2, CentralPanel, Sense, Id, RichText, CursorIcon, Spinner, Label, Layout, ImageSource },
	epaint::{ Rounding, Color32, Stroke, Vec2, FontId, FontFamily, Rect, Pos2, ColorImage, vec2, Rgba },
	IconData, emath::{Align2, Align}
};
use hero_core::{
	Icon,
	uuid::Uuid,
	state::{ State, LoadingBarType },
	parking_lot::RwLock,
	user_interface::Element
};
use poll_promise::Promise;

#[cfg(target_os = "windows")]
use window_shadows::set_shadow;

#[cfg(target_os = "windows")]
use window_vibrancy::apply_mica;

mod blur;
mod widget;
//mod gif_loader;

use widget::{ PathSelect, NavigationItem };

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
	deep_link::prepare("HERO");

	let options = eframe::NativeOptions {
		centered: true,
		renderer: eframe::Renderer::Wgpu,
		resizable: true,
		decorated: false,
		#[cfg(target_os = "windows")]
		icon_data: Some(IconData::try_from_png_bytes(include_bytes!("app_icon.png")).unwrap()),
		#[cfg(target_os = "macos")]
		icon_data: Some(IconData::try_from_png_bytes(include_bytes!("app_icon_macos.png")).unwrap()),
		#[cfg(target_os = "windows")]
		transparent: true,
		initial_window_size: Some(egui::vec2(700., 440.)),
		..Default::default()
	};

	eframe::run_native("HERO", options, Box::new(|cc| {
		egui_extras::install_image_loaders(&cc.egui_ctx);
		//cc.egui_ctx.add_image_loader(Arc::new(gif_loader::GifLoader::default()));
		Box::new(HEROApp::new(cc))
	}))
}

#[derive(Clone, PartialEq)]
enum AppPage {
	SelectGame(bool),
	Home,
	Library,
	Instance(Uuid),
	Extension(String)
}

#[derive(serde::Deserialize)]
struct WebExtension {
	id: String,
	name: String,
	flags: u8,
	avatar_url: Option<String>,
	display_name: String,
	supported_platforms: u8,
	creator: WebExtensionCreator
}

#[derive(serde::Deserialize)]
struct WebExtensionCreator {
	id: String,
	name: String,
	display_name: String
}

struct HEROApp {
	page: Arc<RwLock<AppPage>>,
	images: HashMap<String, egui::Image<'static>>,
	texture: Option<egui::TextureHandle>,
	screenshot: Option<ColorImage>,
	extensions: HashMap<String, Promise<WebExtension>>, 
	instance_page: String,
	instance_banners: HashMap<Uuid, egui::TextureHandle>
}

impl HEROApp {
	fn new(cc: &eframe::CreationContext<'_>) -> Self {
		#[cfg(target_os = "windows")] {
			apply_mica(cc, None).unwrap();
			set_shadow(cc, true).unwrap();
		}

		let ctx = &cc.egui_ctx;
		let mut fonts = egui::FontDefinitions::default();

		for (weight, data) in [
			("100", include_bytes!("font/Inter-Thin.ttf").as_slice()),
			("200", include_bytes!("font/Inter-ExtraLight.ttf").as_slice()),
			("300", include_bytes!("font/Inter-Light.ttf").as_slice()),
			("400", include_bytes!("font/Inter-Regular.ttf").as_slice()),
			("500", include_bytes!("font/Inter-Medium.ttf").as_slice()),
			("600", include_bytes!("font/Inter-SemiBold.ttf").as_slice()),
			("700", include_bytes!("font/Inter-Bold.ttf").as_slice()),
			("800", include_bytes!("font/Inter-ExtraBold.ttf").as_slice()),
			("900", include_bytes!("font/Inter-Black.ttf").as_slice())
		] {
			let name = format!("inter-{}", weight);
			fonts.font_data.insert(
				name.clone(),
				egui::FontData::from_static(data),
			);
			fonts
				.families
				.entry(egui::FontFamily::Name(name.clone().into()))
				.or_default()
				.insert(0, name);
		}

		ctx.set_fonts(fonts);

		let state = State::get();
		state.extensions.write().load_libraries(state.path.join("extensions"));

		hero_core::extension::load_extensions();

		state.localisation.write().insert_data("en-AU", vec![
			("loading.load_game.0", "Loading game information..."),
			("loading.load_game.1", "Loading instances..."),
			("loading.load_game.2", "\\^o^/")
		]);

		let page = Arc::new(RwLock::new(AppPage::SelectGame(false)));

		let p = page.clone();
		if let Err(err) = deep_link::register(
			"hero",
			move |request| {
				if let Some(sublink) = request.strip_prefix("hero://") {
					match sublink.split_once('/') {
						Some(("extension", id)) => {
							*p.write() = AppPage::Extension(id.to_string());
						},
						_ => println!("unrecognised path {}", sublink)
					}
				}
			}
		) {
			println!("deep link fail! {}", err);
		}

		Self {
			page,
			images: HashMap::new(),
			texture: None,
			screenshot: None,
			extensions: HashMap::new(),
			instance_page: "global_instance_info".into(),
			instance_banners: HashMap::new()
		}
	}
}

/*fn ease_in_out(t: f32, p: i32) -> f32 {
	ease_in_out2(t, p, p)
}*/

fn ease_in_out2(t: f32, p1: i32, p2: i32) -> f32 {
	if t <= 0.5 {
		return (t * 2.).powi(p1) / 2.;
	}
	1. - (2. - t * 2.).powi(p2) / 2.
}

impl eframe::App for HEROApp {
	fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
		if cfg!(target_os = "windows") { [0., 0., 0., 0.] } else { [0.05, 0.05, 0.05, 1.] }
	}

	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		let last_page = self.page.read().clone();
		let frame_panel = egui::Frame {
			..Default::default()
		};

		let state = State::get();
		let games = state.get_games();
		for game in games.iter() {
			let key = format!("game_{}", game.id());
			if !self.images.contains_key(&key) {
				if let Some(icon) = game.icon() {
					self.images.insert(key, egui::Image::from_bytes(format!("bytes://{}", icon.path), icon.bytes));
				} else {
					self.images.insert(key, egui::Image::new(egui::include_image!("placeholder.png")));
				}
			}
		}

		for instance in state.instances.read().items.values() {
			let key = format!("instance_{}", instance.id());
			if !self.images.contains_key(&key) {
				if let Some(icon) = instance.icon() {
					self.images.insert(key, egui::Image::from_bytes(format!("bytes://{}", icon.path), icon.bytes).rounding(Rounding::same(8.)));
				} else {
					self.images.insert(key, egui::Image::new(egui::include_image!("placeholder.png")).rounding(Rounding::same(8.)));
				}
			}
		}

		for instance in state.instances.read().items.values() {
			let id = instance.id();
			if !self.instance_banners.contains_key(&id) {
				let image = if let Some(icon) = instance.icon() {
					image::load_from_memory(icon.bytes.as_slice())
				} else { image::load_from_memory(include_bytes!("placeholder.png")) }.unwrap();
				
				let image = blur::blur(8., 8., image);
				let handle = ctx.load_texture(
					format!("instance_banner_{}", id.to_string()),
					ColorImage::from_rgba_unmultiplied(
						[image.width() as _, image.height() as _],
						image.into_rgba8().as_bytes()
					),
					Default::default()
				);
				self.instance_banners.insert(id, handle);
			}
		}

		CentralPanel::default().frame(frame_panel).show(ctx, |ui| {
			if let Some(screenshot) = self.screenshot.take() {
				self.texture = Some(ui.ctx().load_texture(
                    "screenshot",
                    screenshot,
                    Default::default(),
                ));
			}

			let scr_texture = self.texture.as_ref();
			let scr_alpha = ctx.animate_value_with_time(Id::new("screenshot"), scr_texture.map_or(1., |_| 0.), 0.2).powi(5);
			//ctx.set_pixels_per_point(0.95 + scr_alpha * 0.05);

			let page = self.page.read().clone();
			let app_rect = ui.max_rect();
			let content_rect = match page {
				AppPage::SelectGame(_) => app_rect,
				_ => {
					let title_bar_height = 32.0;
					let title_bar_rect = {
						let mut rect = app_rect;
						rect.max.y = rect.min.y + title_bar_height;
						rect
					};
					title_bar_ui(ctx, ui, frame, title_bar_rect, false);

					let mut rect = app_rect;
					rect.min.y += title_bar_height;
					rect
				}
			};

			let mut content_ui = ui.child_ui(content_rect, *ui.layout());
			match page {
				AppPage::SelectGame(loading) => {
					let mut bar_rect = content_rect;
					bar_rect.max.y = 32.;

					egui::Image::new(egui::include_image!("select_background.png"))
						.tint(Color32::from_white_alpha(64))
						.paint_at(&mut content_ui, Rect::from_center_size(content_rect.center(), Vec2::splat(content_rect.max.to_vec2().max_elem())));
					
					let loading_game = state.loading_bars.read().values().find_map(|x| match &x.bar_type {
						LoadingBarType::LoadGame { game_id } => state.get_game(game_id.clone()).map(|g| (g, x.current.clone()))
					});
					if loading_game.is_none() {
						title_bar_ui(ctx, &mut content_ui, frame, bar_rect, true);

						egui::Image::new(egui::include_image!("logo.svg"))
							.paint_at(&mut content_ui, Rect::from_min_max(Pos2::new(40., 32.), Pos2::new(126.4, 56.)));

						if loading {
							*self.page.write() = AppPage::Home;
						}
					}

					let mut rect = content_rect;
					rect.min.y = if loading_game.is_some() { 130. } else { 140. };

					if loading_game.is_some() || !loading {
						content_ui.allocate_ui_at_rect(rect, |ui| ui.vertical_centered(|ui| {
							if let Some((game, progress)) = loading_game {
								Spinner::new()
									.paint_at(ui, Rect::from_center_size(rect.center_top() + Vec2::DOWN * 20., Vec2::splat(40.)));
								ui.painter()
									.text(rect.center_top() + Vec2::DOWN * 60., Align2::CENTER_TOP, state.t(format!("loading.load_game.{}", progress)), FontId::new(18., FontFamily::Name("inter-500".into())), Color32::WHITE);
	
								let rect = Rect::from_center_size(rect.center_top() + Vec2::DOWN * 186. + Vec2::LEFT * 12., Vec2::splat(32.));
								let galley = ui.painter()
									.layout_no_wrap(state.t(format!("game.{}", game.id())), FontId::new(16., FontFamily::Name("inter-400".into())), Color32::WHITE);
								
								let size = galley.size() / 2.;
								let rect2 = rect.translate(Vec2::LEFT * size.x.floor());
								ui.painter()
									.rect_filled(rect2.expand(2.), Rounding::ZERO, Color32::from_white_alpha(2));
								self.images.get(format!("game_{}", game.id()).as_str()).unwrap()
									.paint_at(ui, rect2);
	
								ui.painter()
									.galley(rect2.right_center() + Vec2::UP * size.y + Vec2::RIGHT * 24., galley);
							} else {
								ui.spacing_mut().item_spacing.y = 40.;
								ui.label(
									RichText::new("What are you playing?")
										.color(Color32::WHITE)
										.family(FontFamily::Name("inter-800".into()))
										.size(32.)
								);
								let rect = Rect::from_center_size(rect.center_top() + Vec2::DOWN * 128. + Vec2::LEFT * (48. * (games.len() as f32 - 1.)) + Vec2::LEFT * (16. * (games.len() as f32 - 1.)), Vec2::splat(96.));
								for (index, game) in games.iter().enumerate() {
									let rect = rect.translate(Vec2::RIGHT * (96. * index as f32) + Vec2::RIGHT * (32. * index as f32));
									let rect2 = rect.expand2(Vec2::new(0., 8.));
									let interaction = ui.interact(rect2, Id::new(game.id()), Sense::hover().union(Sense::click()));
									if interaction.clicked() {
										let id = game.id().to_string();
										*self.page.write() = AppPage::SelectGame(true);
	
										tokio::spawn(async move {
											hero_core::state::load_game(id).await;
										});
									}
	
									let target = if interaction.hovered() {
										ui.output_mut(|x| x.cursor_icon = CursorIcon::PointingHand);
										1.
									} else { 0. };
									let alpha = ease_in_out2(ctx.animate_value_with_time(Id::new(game.id()), target, 0.5), 6, 3);
									
									let rect3 = rect.translate(Vec2::new(0., -8. * alpha));
	
									let painter = ui.painter();
									painter.rect(rect3, Rounding::ZERO, Color32::from_white_alpha(1), Stroke::new(2., Color32::from_white_alpha(2 + (alpha * 253.) as u8)));
									
									self.images.get(format!("game_{}", game.id()).as_str()).unwrap()
										.paint_at(ui, rect3.shrink(8.));
	
									if alpha > 0. {
										ui.painter().text(rect.max + Vec2::new(-48., 8.), Align2::CENTER_TOP, state.t(format!("game.{}", game.id())), FontId::new(16., FontFamily::Name("inter-400".into())), Color32::from_white_alpha(((alpha / 0.5).min(1.) * 255.) as u8));
									}
								}
							}
						}));
					}
				},
				AppPage::Extension(id) => {
					content_ui.label(RichText::new(&id).font(FontId::new(16., FontFamily::Name("inter-400".into()))));
					if let Some(promise) = self.extensions.get(&id) {
						if let Some(extension) = promise.ready() {
							let icon_rect = Rect::from_min_size(content_rect.left_top() + vec2(48., 64.), vec2(112., 112.));
							content_ui.painter().rect_filled(icon_rect, Rounding::same(16.), Color32::from_white_alpha(1));
							
							egui::Image::new(ImageSource::Uri(extension.avatar_url.clone().unwrap().into()))
								.paint_at(&mut content_ui, icon_rect.shrink(8.));

							content_ui.painter()
								.text(content_rect.left_top() + vec2(48., 200.), Align2::LEFT_TOP, extension.display_name.clone(), FontId::new(32., FontFamily::Name("inter-700".into())), Color32::WHITE);
						}
					} else {
						self.extensions.insert(id.clone(), Promise::spawn_thread("extension_get", move || {
							reqwest::blocking::get(format!("https://hero-devs.com/api/v1/extension/{}", id))
								.unwrap()
								.json::<WebExtension>()
								.unwrap()
						}));
					}
				},
				_ => {
					let mut rect = content_rect.shrink(16.);
					rect.max.x = rect.min.x + 224.;

					egui::Image::new(egui::include_image!("select_background.png"))
						.tint(Color32::from_white_alpha(16))
						.paint_at(ui, Rect::from_center_size(app_rect.center(), Vec2::splat(app_rect.max.to_vec2().max_elem())));

					content_ui.allocate_ui_at_rect(rect, |ui| {
						ui.vertical(|ui| {
							ui.spacing_mut().item_spacing.y = 4.;

							let page = self.page.read().clone();
							if ui.add(NavigationItem::new("Home", egui::include_image!("icon/house.svg"), matches!(page, AppPage::Home))).clicked() {
								*self.page.write() = AppPage::Home;
							}
							if ui.add(NavigationItem::new("Library", egui::include_image!("icon/collection.svg"), matches!(page, AppPage::Library))).clicked() {
								*self.page.write() = AppPage::Library;
							}
						});

						let mut content_rect = content_rect;
						content_rect.min.x = rect.max.x + 80.;

						let mut ui = ui.child_ui(content_rect, *ui.layout());
						match page {
							AppPage::Home => {
								
							},
							AppPage::Library => {
								ui.add_space(48.);
								ui.label(
									RichText::new("Your Instances")
										.size(32.)
										.color(Color32::WHITE)
										.family(FontFamily::Name("inter-700".into()))
								);
								ui.add_space(24.);
								ui.horizontal(|ui| {
									ui.spacing_mut().item_spacing = Vec2::X * 40.;

									for instance in state.instances.read().items.values() {
										let (rect, response) = ui.allocate_exact_size(vec2(160., 210.), Sense::click().union(Sense::hover()));
										let target = if response.hovered() {
											ui.output_mut(|x| x.cursor_icon = CursorIcon::PointingHand);
											2. / 255.
										} else { 1. / 255. };
										let alpha = ctx.animate_value_with_time(Id::new(format!("instance_{}", instance.id())), target, 0.25);
										ui.painter().rect_filled(rect, Rounding::same(24.), Rgba::from_white_alpha(alpha));
										ui.painter()
											.rect_stroke(rect.shrink(0.5), Rounding::same(24.), Stroke::new(1., Rgba::from_white_alpha(alpha)));

										let inner_rect = rect.shrink(16.);

										let mut img_rect = inner_rect.clone();
										img_rect.max.y = img_rect.min.y + 128.;

										self.images.get(format!("instance_{}", instance.id()).as_str()).unwrap()
											.paint_at(ui, img_rect);

										ui.painter()
											.text(inner_rect.left_top() + Vec2::DOWN * 140., Align2::LEFT_TOP, instance.display_name(), FontId::new(16., FontFamily::Name("inter-500".into())), Color32::WHITE);

										let mut author_rect = inner_rect.clone();
										author_rect.min.y += 164.;

										ui.allocate_ui_at_rect(author_rect, |ui| {
											ui.add(Label::new(
												RichText::new(format!("by {}", instance.metadata.author.clone()))
													.size(13.)
													.color(Color32::from_white_alpha(16))
													.family(FontFamily::Name("inter-400".into()))
											).truncate(true));
										});
										/*ui.painter()
											.text(inner_rect.left_top() + Vec2::DOWN * 164., Align2::LEFT_TOP, format!("by {}", instance.metadata.author.clone()), FontId::new(13., FontFamily::Name("inter-400".into())), Color32::from_white_alpha(64));*/

										if response.clicked() {
											*self.page.write() = AppPage::Instance(instance.id());
										}
									}
								});
							},
							AppPage::Instance(id) => {
								let instances = state.instances.read();
								let instance = instances.items.get(&id).unwrap();
								let mut rect = content_rect.clone();
								rect.max.y = rect.min.y + 192.;

								ui.set_clip_rect(rect);

								let mut rect2 = rect.clone();
								rect2.max.y = rect.min.y + rect.width();
								rect2.set_center(rect.center());

								rect2 = rect2.expand(128.).translate(vec2(-112., 0.));

								egui::Image::new(self.instance_banners.get(&id).unwrap())
									.tint(Color32::from_white_alpha(24))
									.paint_at(&mut ui, rect2);

								ui.set_clip_rect(ui.max_rect());

								let icon_rect = Rect::from_min_size(content_rect.left_top() + vec2(32., 64.), vec2(96., 96.));
								self.images.get(format!("instance_{}", id).as_str())
									.unwrap()
									.paint_at(&mut ui, icon_rect);

								ui.painter()
									.text(content_rect.left_top() + vec2(160., 80.), Align2::LEFT_TOP, instance.display_name(), FontId::new(36., FontFamily::Name("inter-800".into())), Color32::WHITE);

								let launch_rect = Rect::from_center_size(pos2(content_rect.right() - 52., icon_rect.center().y), vec2(40., 40.));
								let response = ui.interact(launch_rect, Id::new("instance_launch"), Sense::click().union(Sense::hover()));
								let target = if response.hovered() {
									ui.output_mut(|x| x.cursor_icon = CursorIcon::PointingHand);
									8.
								} else { 0. };
								let alpha = ctx.animate_value_with_time(Id::new("instance_launch"), target, 0.25) as u8;

								ui.painter()
									.rect_filled(launch_rect, Rounding::same(12.), Color32::from_white_alpha(4 + alpha));
								ui.painter()
									.rect_stroke(launch_rect.shrink(0.5), Rounding::same(12.), Stroke::new(1., Color32::from_white_alpha(4)));

								egui::Image::new(egui::include_image!("icon/play_fill.svg"))
									.paint_at(&mut ui, launch_rect.shrink(8.));

								if response.clicked() {
									tokio::spawn(async move {
										hero_core::instance::launch(id.clone()).await;
									});
								}

								let pages = state.ui_containers.read();

								let mut rect = content_rect.shrink2(vec2(32., 0.));
								rect.min.y += 184.;

								ui.allocate_ui_at_rect(rect, |ui| {
									ui.horizontal(|ui| {
										for page in pages.iter() {
											if ui.add(NavigationItem::new(state.t(format!("page.{}", page.id)), page.icon.clone(), self.instance_page == page.id)).clicked() {
												self.instance_page = page.id.into();
											}
										}
									});
								});
								
								if let Some(page) = pages.iter().find(|x| x.id == self.instance_page) {
									rect.min.y += 44.;

									let mut ui = ui.child_ui(rect, Layout::top_down(Align::LEFT));
									ui.spacing_mut().item_spacing = Vec2::Y * 16.;

									for element in (page.render)() {
										match element {
											Element::Text(text) => {
												ui.label(
													RichText::new(text.read(Some(&instance)).unwrap())
														.size(16.)
														.color(Color32::WHITE)
														.family(FontFamily::Name("inter-400".into()))	
												);
											},
											Element::PathSelect { kind, value } => {
												let mut path = value.read(Some(&instance)).unwrap_or("path not set".into());
												if ui.add(PathSelect::new(kind, &mut path)).changed() {
													value.write(path, Some(instance.id()));
												}
											}
										};
									}
								}
							},
							_ => {
								ui.label(RichText::new("this page is completely empty... how strange!").family(FontFamily::Name("inter-400".into())));
							}
						}
					});
				}
			}

			if let Some(texture) = scr_texture {
				let alpha2 = (1. - scr_alpha) * 0.05;
				egui::Image::new(texture)
					.tint(Color32::from_white_alpha((scr_alpha * 255.) as u8))
					.paint_at(ui, app_rect.shrink2(Vec2::new(app_rect.width() / 2. * alpha2, app_rect.height() / 2. * alpha2)));

				if scr_alpha == 0. {
					self.texture = None;
				}
            }
		});

		let page = self.page.read().clone();
		if page != last_page {
			frame.request_screenshot();
			frame.set_window_size(match page {
				AppPage::SelectGame(_) => Vec2::new(700., 440.),
				_ => Vec2::new(1000., 600.)
			});
			frame.set_centered();
		}
    }

	fn post_rendering(&mut self, _window_size: [u32; 2], frame: &eframe::Frame) {
		if let Some(screenshot) = frame.screenshot() {
			self.screenshot = Some(screenshot);
		}
	}
}

fn title_bar_ui(
	ctx: &egui::Context,
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
    title_bar_rect: eframe::epaint::Rect,
	buttons_only: bool
) {
	let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());
	if title_bar_response.is_pointer_button_down_on() {
        frame.drag_window();
    }

	if !buttons_only {
		ui.painter().rect(title_bar_rect, Rounding::ZERO, Color32::from_white_alpha(1), Stroke::NONE);

		ui.allocate_ui_at_rect(title_bar_rect.shrink2(Vec2::new(16., 10.)), |ui| {
			ui.add(
				egui::Image::new(egui::include_image!("logo.svg"))
					.max_height(12.)
			);
		});
	}

	let mut rect = title_bar_rect;
	rect.min.x = rect.max.x - rect.height();
	
	let response = ui.interact(rect, Id::new("title_bar_close"), Sense::hover().union(Sense::click()));
	if response.clicked() {
		frame.close();
	}
	
	let alpha = ctx.animate_value_with_time(Id::new("title_bar_close"), if response.hovered() { 1. } else { 0. }, 0.15);
	if alpha > 0. {
		ui.painter().rect(rect, Rounding::ZERO, Color32::from_rgba_unmultiplied(226, 42, 39, (alpha * 255.) as u8), Stroke::NONE);
	}

	egui::Image::new(egui::include_image!("close.svg"))
		.tint(Color32::from_white_alpha(((0.1 + alpha * 0.9) * 255.) as u8))
		.paint_at(ui, rect);
}