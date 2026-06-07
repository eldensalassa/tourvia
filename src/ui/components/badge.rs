#![allow(dead_code)]
use egui::{Color32, RichText, Ui, Widget, Response};
use crate::ui::theme;

pub struct Badge {
    text: String,
    bg_color: Color32,
    text_color: Color32,
}

impl Badge {
    pub fn new(text: impl Into<String>, bg_color: Color32, text_color: Color32) -> Self {
        Self {
            text: text.into(),
            bg_color,
            text_color,
        }
    }
    
    pub fn success(text: impl Into<String>) -> Self {
        Self::new(text, theme::SUCCESS().linear_multiply(0.15), theme::SUCCESS())
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self::new(text, theme::WARNING().linear_multiply(0.15), theme::WARNING())
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self::new(text, theme::ERROR().linear_multiply(0.15), theme::ERROR())
    }
    
    pub fn info(text: impl Into<String>) -> Self {
        Self::new(text, theme::INFO().linear_multiply(0.15), theme::INFO())
    }
    
    pub fn neutral(text: impl Into<String>) -> Self {
        Self::new(text, theme::BG_ELEVATED(), theme::TEXT_SECONDARY())
    }
}

impl Widget for Badge {
    fn ui(self, ui: &mut Ui) -> Response {
        let text = RichText::new(self.text).size(11.0).color(self.text_color).strong();
        let padding = egui::Margin::symmetric(10, 4);
        
        egui::Frame::new()
            .fill(self.bg_color)
            .corner_radius(theme::badge_rounding())
            .inner_margin(padding)
            .show(ui, |ui| {
                ui.label(text)
            })
            .response
    }
}
