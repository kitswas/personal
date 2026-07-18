use eframe::egui;

pub trait BadgeUiExtensions {
    fn badge(&mut self, text: &str) -> egui::Response;
    fn badge_success(&mut self, text: &str) -> egui::Response;
}

impl BadgeUiExtensions for egui::Ui {
    fn badge(&mut self, text: &str) -> egui::Response {
        let btn = egui::Button::new(egui::RichText::new(text).size(12.0))
            .fill(egui::Color32::from_rgb(244, 244, 245)) // secondary
            .stroke(egui::Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(12));
        self.add_enabled(false, btn)
    }
    
    fn badge_success(&mut self, text: &str) -> egui::Response {
        let btn = egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE).size(12.0))
            .fill(egui::Color32::from_rgb(0, 128, 50)) // success
            .stroke(egui::Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(12));
        self.add_enabled(false, btn)
    }
}
