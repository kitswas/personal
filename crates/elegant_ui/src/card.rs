use crate::theme::ElegantTheme;
use egui;

#[derive(Default)]
pub struct Card {}

impl Card {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn show<R>(
		self,
		ui: &mut egui::Ui,
		add_contents: impl FnOnce(&mut egui::Ui) -> R,
	) -> egui::InnerResponse<R> {
		let theme = ElegantTheme::get(ui.ctx());

		egui::Frame::new()
            .fill(theme.secondary.linear_multiply(0.3)) // faint background
            .stroke(egui::Stroke::new(theme.spacing.border_width, theme.border))
            .inner_margin(theme.spacing.card_inner_margin)
            .corner_radius(egui::CornerRadius::same(theme.spacing.corner_radius as u8))
            // FIX: Just pass the UI directly. Do not calculate available_width here.
            .show(ui, add_contents)
	}
}
