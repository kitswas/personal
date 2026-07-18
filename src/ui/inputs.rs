use eframe::egui;

pub trait InputUiExtensions {
    fn text_input(&mut self, text: &mut String, placeholder: &str) -> egui::Response;
}

impl InputUiExtensions for egui::Ui {
    fn text_input(&mut self, text: &mut String, placeholder: &str) -> egui::Response {
        let frame = egui::Frame::new()
            .inner_margin(egui::vec2(12.0, 8.0))
            .corner_radius(egui::CornerRadius::same(6))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(212, 212, 216))) // --border
            .fill(egui::Color32::WHITE);
            
        frame.show(self, |ui| {
            ui.add_sized(
                egui::vec2(ui.available_width(), 16.0),
                egui::TextEdit::singleline(text)
                    .hint_text(placeholder)
            )
        }).inner
    }
}
