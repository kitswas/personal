use eframe::egui;
use egui_elegant::{ElegantFont, ElegantTheme, ThemeMode};

fn main() -> eframe::Result<()> {
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 400.0]),
		..Default::default()
	};

	eframe::run_native(
		"Noto Multilingual Example",
		options,
		Box::new(|cc| {
			let theme = ElegantTheme::build(ThemeMode::System, ElegantFont::Noto);
			theme.apply(&cc.egui_ctx);
			Ok(Box::new(MultilingualApp {}))
		}),
	)
}

struct MultilingualApp {}

impl eframe::App for MultilingualApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ui, |ui| {
			ui.vertical_centered(|ui| {
				ui.add_space(40.0);
				ui.heading(
					egui::RichText::new("Noto Fonts Multilingual Example").size(32.0),
				);
				ui.add_space(20.0);
			});

			ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
				ui.label(egui::RichText::new("English: Hello World").size(24.0));
				ui.add_space(10.0);
				ui.label(egui::RichText::new("Hindi (हिन्दी): नमस्ते").size(24.0));
				ui.add_space(10.0);
				ui.label(egui::RichText::new("Bengali (বাংলা): নমস্কার").size(24.0));
				ui.add_space(20.0);

				ui.label(
					egui::RichText::new("Monospace Fallback:")
						.family(egui::FontFamily::Monospace)
						.size(18.0),
				);
				ui.label(
					egui::RichText::new("fn main() { println!(\"Hello\"); }")
						.family(egui::FontFamily::Monospace)
						.size(16.0),
				);
			});
		});
	}
}
