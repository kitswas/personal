use crate::{theme::ElegantTheme, traits::Elegant};
use egui;

pub struct ElegantTabs<'a> {
	tabs: &'a [&'a str],
	selected: &'a mut usize,
}

impl<'a> ElegantTabs<'a> {
	pub fn new(tabs: &'a [&'a str], selected: &'a mut usize) -> Self {
		Self { tabs, selected }
	}
}

impl<'a> egui::Widget for ElegantTabs<'a> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let theme = ElegantTheme::get(ui.ctx());

		let mut changed = false;
		let mut response: Option<egui::Response> = None;

		ui.horizontal(|ui| {
			ui.spacing_mut().item_spacing.x = 24.0; // Spacing between tabs

			for (i, &tab) in self.tabs.iter().enumerate() {
				let is_selected = *self.selected == i;

				let (rect, mut current_response) = ui.allocate_exact_size(
					egui::vec2(
						ui.painter()
							.layout_no_wrap(
								tab.to_string(),
								egui::FontId::proportional(16.0),
								theme.foreground,
							)
							.size()
							.x,
						32.0,
					),
					egui::Sense::click(),
				);

				if current_response.clicked() {
					*self.selected = i;
					changed = true;
					current_response.mark_changed();
				}

				if current_response.hovered() {
					ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
				}

				let text_color = if is_selected {
					theme.foreground
				} else if current_response.hovered() {
					theme.foreground.linear_multiply(0.8)
				} else {
					theme.foreground.linear_multiply(0.6)
				};

				ui.painter().text(
					rect.center(),
					egui::Align2::CENTER_CENTER,
					tab,
					egui::FontId::proportional(16.0),
					text_color,
				);

				if is_selected {
					let line_y = rect.max.y - 2.0;
					ui.painter().line_segment(
						[
							egui::pos2(rect.min.x, line_y),
							egui::pos2(rect.max.x, line_y),
						],
						egui::Stroke::new(2.0, theme.primary),
					);
				}

				if let Some(r) = response.as_mut() {
					*r = r.union(current_response);
				} else {
					response = Some(current_response);
				}
			}
		});

		let mut res = response.unwrap_or_else(|| {
			ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
		});
		if changed {
			res.mark_changed();
		}
		res
	}
}

impl<'a> Elegant for ElegantTabs<'a> {}
crate::impl_flex_widget!(ElegantTabs<'a>);
