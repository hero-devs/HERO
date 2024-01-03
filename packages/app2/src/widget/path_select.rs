
use eframe::{
	egui::{ Ui, Response, Sense, Widget, CursorIcon },
	emath::Align2,
	epaint::{ Vec2, Rounding, Color32, FontId, FontFamily }
};
use hero_core::user_interface::PathSelectKind;

pub struct PathSelect<'t> {
	kind: PathSelectKind,
	value: &'t mut String
}

impl<'t> PathSelect<'t> {
	pub fn new(kind: PathSelectKind, value: &'t mut String) -> Self {
		Self {
			kind,
			value
		}
	}
}

impl<'t> Widget for PathSelect<'t> {
	fn ui(self, ui: &mut Ui) -> Response {
		let (rect, mut response) = ui.allocate_at_least(Vec2::new(ui.available_width(), 32.), Sense::click().union(Sense::hover()));
		if ui.is_rect_visible(rect) {
			let colour = Color32::from_white_alpha(32);
			let mut rect2 = rect.clone();
			rect2.max.x = rect.min.x + 96.;

			let response2 = ui.allocate_rect(rect2, Sense::click().union(Sense::hover()));
			let alpha = if response2.hovered() {
				ui.output_mut(|x| x.cursor_icon = CursorIcon::PointingHand);
				2
			} else { 1 };
			if response2.clicked() {
				let dialog = rfd::FileDialog::new();
				if let Some(path) = match self.kind {
					PathSelectKind::File => dialog.pick_file(),
					PathSelectKind::Directory => dialog.pick_folder()
				} {
					*self.value = path.to_string_lossy().to_string();
					response.mark_changed();
				}
			}

			ui.painter().rect_filled(rect2, Rounding { nw: 16., sw: 16., ne: 0., se: 0. }, Color32::from_white_alpha(alpha));
			ui.painter().text(rect2.center(), Align2::CENTER_CENTER, "Browse...", FontId::new(14., FontFamily::Name("inter-400".into())), Color32::WHITE);

			let mut rect3 = rect.clone();
			rect3.min.x = rect2.max.x + 2.;

			ui.painter().rect_filled(rect3, Rounding { nw: 0., sw: 0., ne: 16., se: 16. }, Color32::from_white_alpha(1));
			ui.painter().text(rect3.left_center() + Vec2::RIGHT * 16., Align2::LEFT_CENTER, self.value, FontId::new(12., FontFamily::Name("inter-400".into())), colour);
		}

		response
	}
}