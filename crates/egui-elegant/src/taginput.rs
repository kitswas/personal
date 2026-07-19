use crate::{badge::ElegantBadge, theme::Variant, traits::Elegant};
use egui;

pub struct ElegantTagInput<'a> {
	tags: &'a mut Vec<String>,
	current_text: &'a mut String,
}

impl<'a> ElegantTagInput<'a> {
	pub fn new(tags: &'a mut Vec<String>, current_text: &'a mut String) -> Self {
		Self { tags, current_text }
	}
}

impl<'a> egui::Widget for ElegantTagInput<'a> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let mut response: Option<egui::Response> = None;

		ui.horizontal_wrapped(|ui| {
			let mut to_remove = None;
			for (i, tag) in self.tags.iter().enumerate() {
				if ui
					.add(ElegantBadge::new(tag).variant(Variant::Secondary))
					.clicked()
				{
					to_remove = Some(i);
				}
			}

			if let Some(i) = to_remove {
				self.tags.remove(i);
			}

			let input_resp = ui.add(
				egui::TextEdit::singleline(self.current_text)
					.hint_text("Add a tag...")
					.margin(egui::vec2(8.0, 8.0)),
			);

			if input_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
				let text = self.current_text.trim();
				if !text.is_empty() {
					self.tags.push(text.to_string());
					self.current_text.clear();
					input_resp.request_focus();
				}
			}

			response = Some(input_resp);
		});

		// Render the bounding box manually if needed, but egui's wrapping horizontal
		// layout works decently. We'll just return the TextEdit's response.
		response.unwrap()
	}
}

impl<'a> Elegant for ElegantTagInput<'a> {}
crate::impl_flex_widget!(ElegantTagInput<'a>);
