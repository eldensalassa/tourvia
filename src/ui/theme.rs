use egui::{Color32, FontId, RichText, Stroke, CornerRadius, Vec2, FontFamily};

// ─── Color Palette (Black / Bronze Theme) ────────────────
pub const BG_DARK: Color32 = Color32::from_rgb(10, 10, 10);      // Pure Black-ish
pub const BG_PANEL: Color32 = Color32::from_rgb(18, 18, 18);     // Very Dark Gray
pub const BG_CARD: Color32 = Color32::from_rgb(26, 26, 26);      // Dark Gray
pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(38, 38, 38);
pub const BG_INPUT: Color32 = Color32::from_rgb(22, 22, 22);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(34, 34, 34);

// Bronze & Gold Accents
pub const ACCENT_BRONZE: Color32 = Color32::from_rgb(205, 127, 50); // Classic Bronze
pub const ACCENT_BRONZE_LIGHT: Color32 = Color32::from_rgb(224, 155, 89);
pub const ACCENT_BRONZE_DARK: Color32 = Color32::from_rgb(138, 86, 34);
pub const GOLD: Color32 = Color32::from_rgb(255, 215, 0);

// Status Colors
pub const SUCCESS: Color32 = Color32::from_rgb(46, 139, 87);     // Sea Green
pub const WARNING: Color32 = Color32::from_rgb(218, 165, 32);    // Goldenrod
pub const ERROR: Color32 = Color32::from_rgb(178, 34, 34);       // Firebrick
pub const INFO: Color32 = Color32::from_rgb(70, 130, 180);       // Steel Blue

// Text Colors
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(235, 235, 235);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(170, 170, 170);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 100, 100);

// Borders
pub const BORDER: Color32 = Color32::from_rgb(50, 50, 50);
pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(35, 35, 35);
pub const CONNECTOR_LINE: Color32 = Color32::from_rgb(80, 80, 80);

// Match status colors
pub const MATCH_PENDING: Color32 = Color32::from_rgb(26, 26, 26);
pub const MATCH_IN_PROGRESS: Color32 = Color32::from_rgb(45, 35, 20); // Dark Bronze tint
pub const MATCH_COMPLETED: Color32 = Color32::from_rgb(20, 35, 25);   // Dark Green tint
pub const MATCH_BYE: Color32 = Color32::from_rgb(40, 40, 40);

// ─── Typography Helpers ─────────────────────────────

pub fn heading_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(22.0, FontFamily::Proportional))
        .color(TEXT_PRIMARY)
        .strong()
}

pub fn subheading_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(16.0, FontFamily::Proportional))
        .color(TEXT_PRIMARY)
        .strong()
}

pub fn body_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(14.0, FontFamily::Proportional))
        .color(TEXT_PRIMARY)
}

pub fn label_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(13.0, FontFamily::Proportional))
        .color(TEXT_SECONDARY)
}

pub fn small_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(11.0, FontFamily::Proportional))
        .color(TEXT_MUTED)
}

pub fn champion_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(20.0, FontFamily::Proportional))
        .color(GOLD)
        .strong()
}

pub fn section_header(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(13.0, FontFamily::Proportional))
        .color(ACCENT_BRONZE_LIGHT)
        .strong()
}

// ─── Widget Style Helpers ───────────────────────────

pub fn card_rounding() -> CornerRadius {
    CornerRadius::same(6) // Challonge is a bit more square
}

pub fn button_rounding() -> CornerRadius {
    CornerRadius::same(4)
}

pub fn card_stroke() -> Stroke {
    Stroke::new(1.0, BORDER)
}

/// Apply the Tourvia Black/Bronze theme to an egui context.
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = BG_PANEL;
    visuals.window_fill = BG_ELEVATED;
    visuals.extreme_bg_color = BG_DARK;
    visuals.faint_bg_color = BG_CARD;

    // Widget styles
    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.inactive.corner_radius = button_rounding();
    visuals.widgets.inactive.weak_bg_fill = BG_CARD;

    visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.corner_radius = button_rounding();

    visuals.widgets.active.bg_fill = ACCENT_BRONZE;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, BG_DARK);
    visuals.widgets.active.corner_radius = button_rounding();

    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);

    visuals.widgets.open.bg_fill = BG_ELEVATED;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, ACCENT_BRONZE);

    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(205, 127, 50, 40); // Bronze transparent
    visuals.selection.stroke = Stroke::new(1.0, ACCENT_BRONZE);

    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_corner_radius = CornerRadius::same(8);

    visuals.striped = true;

    style.visuals = visuals;

    // Spacing
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(16);
    style.spacing.button_padding = Vec2::new(14.0, 7.0);
    style.spacing.indent = 18.0;

    ctx.set_style(style);
}
