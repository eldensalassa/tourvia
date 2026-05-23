use egui::{Color32, FontId, RichText, Stroke, CornerRadius, Vec2, FontFamily};
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Dark,
    Light,
    Custom,
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeConfig {
    pub mode: ThemeMode,
    pub bg_dark: Color32,
    pub bg_panel: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub bg_input: Color32,
    pub bg_elevated: Color32,
    pub accent: Color32,
    pub accent_light: Color32,
    pub accent_dark: Color32,
    pub gold: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub border: Color32,
    pub border_subtle: Color32,
    pub connector_line: Color32,
    pub match_pending: Color32,
    pub match_in_progress: Color32,
    pub match_completed: Color32,
    pub match_bye: Color32,
}

impl ThemeConfig {
    pub const fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            bg_dark: Color32::from_rgb(9, 9, 11),       // Very dark zinc
            bg_panel: Color32::from_rgb(24, 24, 27),    // Dark zinc
            bg_card: Color32::from_rgb(39, 39, 42),     // Elevated zinc
            bg_card_hover: Color32::from_rgb(63, 63, 70),
            bg_input: Color32::from_rgb(24, 24, 27),    
            bg_elevated: Color32::from_rgb(45, 45, 48),
            accent: Color32::from_rgb(197, 160, 89),    // Refined Bronze
            accent_light: Color32::from_rgb(230, 201, 142), // Lighter bronze
            accent_dark: Color32::from_rgb(138, 106, 45),
            gold: Color32::from_rgb(212, 175, 55),
            success: Color32::from_rgb(16, 185, 129),   // Emerald green
            warning: Color32::from_rgb(245, 158, 11),   // Amber
            error: Color32::from_rgb(239, 68, 68),      // Red
            info: Color32::from_rgb(59, 130, 246),      // Blue
            text_primary: Color32::from_rgb(250, 250, 250), // Off-white
            text_secondary: Color32::from_rgb(161, 161, 170), // Zinc-400
            text_muted: Color32::from_rgb(113, 113, 122),   // Zinc-500
            border: Color32::from_rgb(63, 63, 70),      // Zinc-700
            border_subtle: Color32::from_rgb(39, 39, 42), // Zinc-800
            connector_line: Color32::from_rgb(82, 82, 91), // Zinc-600
            match_pending: Color32::from_rgb(39, 39, 42),
            match_in_progress: Color32::from_rgb(66, 50, 25), // Tinted bronze
            match_completed: Color32::from_rgb(25, 40, 30),
            match_bye: Color32::from_rgb(63, 63, 70),
        }
    }

    pub const fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            bg_dark: Color32::from_rgb(240, 240, 240),
            bg_panel: Color32::from_rgb(250, 250, 250),
            bg_card: Color32::from_rgb(255, 255, 255),
            bg_card_hover: Color32::from_rgb(245, 245, 245),
            bg_input: Color32::from_rgb(255, 255, 255),
            bg_elevated: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(0, 122, 204), // Blue accent for light mode
            accent_light: Color32::from_rgb(51, 153, 255),
            accent_dark: Color32::from_rgb(0, 82, 136),
            gold: Color32::from_rgb(218, 165, 32),
            success: Color32::from_rgb(34, 139, 34),
            warning: Color32::from_rgb(205, 133, 63),
            error: Color32::from_rgb(220, 20, 60),
            info: Color32::from_rgb(70, 130, 180),
            text_primary: Color32::from_rgb(20, 20, 20),
            text_secondary: Color32::from_rgb(80, 80, 80),
            text_muted: Color32::from_rgb(120, 120, 120),
            border: Color32::from_rgb(210, 210, 210),
            border_subtle: Color32::from_rgb(230, 230, 230),
            connector_line: Color32::from_rgb(180, 180, 180),
            match_pending: Color32::from_rgb(255, 255, 255),
            match_in_progress: Color32::from_rgb(230, 245, 255),
            match_completed: Color32::from_rgb(230, 255, 230),
            match_bye: Color32::from_rgb(240, 240, 240),
        }
    }
}

pub static ACTIVE_THEME: RwLock<ThemeConfig> = RwLock::new(ThemeConfig::dark());

pub fn set_theme(config: ThemeConfig) {
    if let Ok(mut t) = ACTIVE_THEME.write() {
        *t = config;
    }
}

pub fn get_theme() -> ThemeConfig {
    *ACTIVE_THEME.read().unwrap()
}

// ─── Getters for UI Macros/Code ─────────────────────
pub fn BG_DARK() -> Color32 { get_theme().bg_dark }
pub fn BG_PANEL() -> Color32 { get_theme().bg_panel }
pub fn BG_CARD() -> Color32 { get_theme().bg_card }
pub fn BG_CARD_HOVER() -> Color32 { get_theme().bg_card_hover }
pub fn BG_INPUT() -> Color32 { get_theme().bg_input }
pub fn BG_ELEVATED() -> Color32 { get_theme().bg_elevated }
pub fn ACCENT_BRONZE() -> Color32 { get_theme().accent }
pub fn ACCENT_BRONZE_LIGHT() -> Color32 { get_theme().accent_light }
pub fn ACCENT_BRONZE_DARK() -> Color32 { get_theme().accent_dark }
pub fn GOLD() -> Color32 { get_theme().gold }
pub fn SUCCESS() -> Color32 { get_theme().success }
pub fn WARNING() -> Color32 { get_theme().warning }
pub fn ERROR() -> Color32 { get_theme().error }
pub fn INFO() -> Color32 { get_theme().info }
pub fn TEXT_PRIMARY() -> Color32 { get_theme().text_primary }
pub fn TEXT_SECONDARY() -> Color32 { get_theme().text_secondary }
pub fn TEXT_MUTED() -> Color32 { get_theme().text_muted }
pub fn BORDER() -> Color32 { get_theme().border }
pub fn BORDER_SUBTLE() -> Color32 { get_theme().border_subtle }
pub fn CONNECTOR_LINE() -> Color32 { get_theme().connector_line }
pub fn MATCH_PENDING() -> Color32 { get_theme().match_pending }
pub fn MATCH_IN_PROGRESS() -> Color32 { get_theme().match_in_progress }
pub fn MATCH_COMPLETED() -> Color32 { get_theme().match_completed }
pub fn MATCH_BYE() -> Color32 { get_theme().match_bye }

// ─── Typography Helpers ─────────────────────────────

pub fn heading_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(28.0, FontFamily::Proportional))
        .color(TEXT_PRIMARY())
        .strong()
}

pub fn subheading_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(20.0, FontFamily::Proportional))
        .color(TEXT_PRIMARY())
        .strong()
}

pub fn body_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(16.0, FontFamily::Proportional))
        .color(TEXT_PRIMARY())
}

pub fn label_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(14.0, FontFamily::Proportional))
        .color(TEXT_SECONDARY())
}

pub fn small_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(12.0, FontFamily::Proportional))
        .color(TEXT_MUTED())
}

pub fn champion_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(24.0, FontFamily::Proportional))
        .color(GOLD())
        .strong()
}

pub fn section_header(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(14.0, FontFamily::Proportional))
        .color(ACCENT_BRONZE_LIGHT())
        .strong()
}

// ─── Styling Helpers ─────────────────────────────────
pub fn card_rounding() -> CornerRadius {
    CornerRadius::same(12) // Modern, larger rounding
}

pub fn button_rounding() -> CornerRadius {
    CornerRadius::same(8) // Modern button rounding
}

pub fn card_stroke() -> Stroke {
    Stroke::new(1.0, get_theme().border_subtle)
}

/// Apply the Tourvia theme to an egui context.
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let theme = get_theme();
    
    let mut visuals = if theme.mode == ThemeMode::Dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    visuals.panel_fill = theme.bg_panel;
    visuals.window_fill = theme.bg_elevated;
    visuals.extreme_bg_color = theme.bg_dark;
    visuals.faint_bg_color = theme.bg_card;

    // Widget styles
    visuals.widgets.inactive.bg_fill = theme.bg_card;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text_secondary);
    visuals.widgets.inactive.corner_radius = button_rounding();
    visuals.widgets.inactive.weak_bg_fill = theme.bg_card;

    visuals.widgets.hovered.bg_fill = theme.bg_card_hover;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.text_primary);
    visuals.widgets.hovered.corner_radius = button_rounding();

    visuals.widgets.active.bg_fill = theme.accent;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, theme.bg_dark);
    visuals.widgets.active.corner_radius = button_rounding();

    visuals.widgets.noninteractive.bg_fill = theme.bg_panel;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text_primary);

    visuals.widgets.open.bg_fill = theme.bg_elevated;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, theme.accent);

    visuals.selection.bg_fill = theme.accent; // Or partially transparent
    visuals.selection.stroke = Stroke::new(1.0, theme.accent);

    visuals.window_stroke = Stroke::new(1.0, theme.border);
    visuals.window_corner_radius = CornerRadius::same(12);

    visuals.striped = true;

    style.visuals = visuals;

    // Spacing
    style.spacing.item_spacing = Vec2::new(12.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(20);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    style.spacing.interact_size = Vec2::new(48.0, 32.0);
    style.spacing.indent = 18.0;

    ctx.set_style(style);
}
