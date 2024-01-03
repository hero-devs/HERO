
use eframe::{
	egui::{ Ui, Image, Response, Sense, Widget, CursorIcon, ImageSource },
	emath::Align2,
	epaint::{ Vec2, Rect, Rounding, Color32, FontId, FontFamily }
};

pub struct NavigationItem<'a> {
	text: String,
	icon: ImageSource<'a>,
	active: bool
}

impl<'a> NavigationItem<'a> {
	pub fn new(text: impl ToString, icon: impl Into<ImageSource<'a>>, active: bool) -> Self {
		Self {
			text: text.to_string(),
			icon: icon.into(),
			active
		}
	}
}

impl<'a> Widget for NavigationItem<'a> {
	fn ui(self, ui: &mut Ui) -> Response {
		let (rect, response) = ui.allocate_at_least(Vec2::new(256., 40.), Sense::click().union(Sense::hover()));
		if ui.is_rect_visible(rect) {
			if response.hovered() {
				ui.output_mut(|x| x.cursor_icon = CursorIcon::PointingHand);
			};

			let colour = if self.active {
				ui.painter().rect_filled(rect, Rounding::same(20.), Color32::from_white_alpha(1));
				Color32::WHITE
			} else { Color32::from_white_alpha(32) };
			Image::new(self.icon)
				.tint(colour)
				.paint_at(ui, Rect::from_center_size(rect.left_center() + Vec2::RIGHT * 33., Vec2::splat(18.)));
			
			ui.painter().text(rect.left_center() + Vec2::RIGHT * 56., Align2::LEFT_CENTER, self.text, FontId::new(14., FontFamily::Name("inter-500".into())), colour);
		}
		
		response
	}
}