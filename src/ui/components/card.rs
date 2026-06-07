#![allow(dead_code)]
use egui::{Color32, Response, Sense, Stroke, Ui, Vec2};
use crate::ui::theme;

pub struct Card<'a> {
    size: Vec2,
    content: Box<dyn FnOnce(&mut Ui) + 'a>,
    is_interactive: bool,
    is_selected: bool,
    accessibility_label: Option<String>,
    padding: egui::Margin,
}

impl<'a> Card<'a> {
    pub fn new(size: Vec2, content: impl FnOnce(&mut Ui) + 'a) -> Self {
        Self {
            size,
            content: Box::new(content),
            is_interactive: true,
            is_selected: false,
            accessibility_label: None,
            padding: egui::Margin::same(16),
        }
    }

    pub fn non_interactive(mut self) -> Self {
        self.is_interactive = false;
        self
    }
    
    pub fn is_selected(mut self, val: bool) -> Self {
        self.is_selected = val;
        self
    }
    
    pub fn accessibility_label(mut self, label: impl Into<String>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }
    
    pub fn inner_margin(mut self, margin: impl Into<egui::Margin>) -> Self {
        self.padding = margin.into();
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let sense = if self.is_interactive { Sense::click() } else { Sense::hover() };
        
        let (rect, mut response) = ui.allocate_exact_size(self.size, sense);
        
        if let Some(label) = self.accessibility_label {
            response = response.on_hover_text(label);
        }

        let is_hovered = response.hovered();
        let has_focus = response.has_focus() || self.is_selected;

        // Animate hover state (0.0 to 1.0)
        let hover_factor = ui.ctx().animate_bool(response.id, is_hovered || has_focus);
        
        let bg = if self.is_selected {
            theme::BG_ELEVATED()
        } else {
            lerp_color(theme::BG_CARD(), theme::BG_CARD_HOVER(), hover_factor)
        };
        let border = if has_focus {
            theme::BORDER_FOCUS()
        } else {
            lerp_color(theme::BORDER_SUBTLE(), theme::BORDER_FOCUS(), hover_factor)
        };
        
        let stroke_w = if self.is_selected { 2.0 } else { egui::lerp(1.0..=2.0, hover_factor) };
        
        // Shadow animation - neon glow effect for esports feel
        let blur = egui::lerp(16.0..=32.0, hover_factor);
        let offset_y = egui::lerp(4.0..=0.0, hover_factor); // Centered glow on hover
        let alpha = egui::lerp(
            if theme::get_theme().mode == theme::ThemeMode::Dark { 140.0..=200.0 } else { 20.0..=40.0 }, 
            hover_factor
        );
        let shadow_color = if has_focus {
            // Tint shadow with accent color for a subtle "neon glow"
            let c = theme::ACCENT_BRONZE();
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (alpha * 0.5) as u8)
        } else {
            Color32::from_black_alpha(alpha as u8)
        };

        let shadow = egui::epaint::Shadow {
            offset: [0, offset_y as i8],
            blur: blur as u8,
            spread: if has_focus { 2 } else { 0 },
            color: shadow_color,
        };

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |card_ui| {
            egui::Frame::new()
                .fill(bg)
                .stroke(Stroke::new(stroke_w, border))
                .corner_radius(theme::card_rounding())
                .shadow(shadow)
                .inner_margin(self.padding)
                .show(card_ui, |inner_ui| {
                    inner_ui.set_min_size(rect.size() - self.padding.sum());
                    (self.content)(inner_ui);
                });
        });
        
        response
    }
}

// Helper to lerp colors
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let a_rgba = egui::Rgba::from(a);
    let b_rgba = egui::Rgba::from(b);
    let lerped = a_rgba * (1.0 - t) + b_rgba * t;
    lerped.into()
}
