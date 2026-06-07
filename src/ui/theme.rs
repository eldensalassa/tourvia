#![allow(non_snake_case, dead_code)]

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
    // Surfaces
    pub bg_dark: Color32,
    pub bg_panel: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub bg_input: Color32,
    pub bg_elevated: Color32,
    // Accents
    pub accent: Color32,
    pub accent_light: Color32,
    pub accent_dark: Color32,
    pub gold: Color32,
    // Semantics
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,
    // Typography
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    // Borders
    pub border: Color32,
    pub border_subtle: Color32,
    pub border_focus: Color32,
    pub connector_line: Color32,
    // Match specific states
    pub match_pending: Color32,
    pub match_in_progress: Color32,
    pub match_completed: Color32,
    pub match_bye: Color32,
}

impl ThemeConfig {
    pub const fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            bg_dark: Color32::from_rgb(15, 25, 35),       // #0F1923 (Deep Esports Charcoal)
            bg_panel: Color32::from_rgb(20, 25, 30),      // Slightly lighter panel
            bg_card: Color32::from_rgb(31, 35, 38),       // #1F2326
            bg_card_hover: Color32::from_rgb(43, 47, 50), // #2B2F32
            bg_input: Color32::from_rgb(20, 25, 30),    
            bg_elevated: Color32::from_rgb(31, 35, 38), 
            accent: Color32::from_rgb(229, 168, 83),    // Vibrant Premium Bronze (kept as requested)
            accent_light: Color32::from_rgb(245, 203, 138),
            accent_dark: Color32::from_rgb(173, 115, 33),
            gold: Color32::from_rgb(255, 215, 0),
            success: Color32::from_rgb(52, 211, 153),   // emerald-400
            warning: Color32::from_rgb(251, 191, 36),   // amber-400
            error: Color32::from_rgb(255, 70, 85),      // #FF4655 (Valorant Red)
            info: Color32::from_rgb(96, 165, 250),      // blue-400
            text_primary: Color32::from_rgb(250, 250, 250), // zinc-50
            text_secondary: Color32::from_rgb(161, 161, 170), // zinc-400
            text_muted: Color32::from_rgb(113, 113, 122),   // zinc-500
            border: Color32::from_rgb(82, 82, 91),      // zinc-600
            border_subtle: Color32::from_rgb(63, 63, 70), // zinc-700
            border_focus: Color32::from_rgb(229, 168, 83),
            connector_line: Color32::from_rgb(100, 100, 110), 
            match_pending: Color32::from_rgb(31, 35, 38),
            match_in_progress: Color32::from_rgb(89, 65, 32),
            match_completed: Color32::from_rgb(32, 60, 45),
            match_bye: Color32::from_rgb(43, 47, 50),
        }
    }

    pub const fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            bg_dark: Color32::from_rgb(244, 244, 245), // zinc-100
            bg_panel: Color32::from_rgb(250, 250, 250), // zinc-50
            bg_card: Color32::from_rgb(255, 255, 255), // white
            bg_card_hover: Color32::from_rgb(244, 244, 245), // zinc-100
            bg_input: Color32::from_rgb(255, 255, 255),
            bg_elevated: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(217, 138, 30), // Darker bronze for light mode
            accent_light: Color32::from_rgb(235, 173, 85),
            accent_dark: Color32::from_rgb(150, 90, 15),
            gold: Color32::from_rgb(218, 165, 32),
            success: Color32::from_rgb(5, 150, 105), // emerald-600
            warning: Color32::from_rgb(217, 119, 6), // amber-600
            error: Color32::from_rgb(220, 38, 38), // red-600
            info: Color32::from_rgb(37, 99, 235), // blue-600
            text_primary: Color32::from_rgb(24, 24, 27), // zinc-900
            text_secondary: Color32::from_rgb(82, 82, 91), // zinc-600
            text_muted: Color32::from_rgb(161, 161, 170), // zinc-400
            border: Color32::from_rgb(212, 212, 216), // zinc-300
            border_subtle: Color32::from_rgb(228, 228, 231), // zinc-200
            border_focus: Color32::from_rgb(217, 138, 30),
            connector_line: Color32::from_rgb(161, 161, 170),
            match_pending: Color32::from_rgb(255, 255, 255),
            match_in_progress: Color32::from_rgb(254, 243, 199),
            match_completed: Color32::from_rgb(209, 250, 229),
            match_bye: Color32::from_rgb(244, 244, 245),
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
pub fn BORDER_FOCUS() -> Color32 { get_theme().border_focus }
pub fn CONNECTOR_LINE() -> Color32 { get_theme().connector_line }
pub fn MATCH_PENDING() -> Color32 { get_theme().match_pending }
pub fn MATCH_IN_PROGRESS() -> Color32 { get_theme().match_in_progress }
pub fn MATCH_COMPLETED() -> Color32 { get_theme().match_completed }
pub fn MATCH_BYE() -> Color32 { get_theme().match_bye }

// ─── Typography Helpers ─────────────────────────────

pub fn heading_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(28.0, FontFamily::Name("Impact".into())))
        .color(TEXT_PRIMARY())
        .strong()
}

pub fn subheading_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::new(20.0, FontFamily::Name("Impact".into())))
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
        .font(FontId::new(24.0, FontFamily::Name("Impact".into())))
        .color(GOLD())
        .strong()
}

pub fn section_header(text: &str) -> RichText {
    RichText::new(text.to_uppercase())
        .font(FontId::new(14.0, FontFamily::Name("Impact".into())))
        .color(ACCENT_BRONZE_LIGHT())
        .strong()
}

// ─── Styling Helpers ─────────────────────────────────
pub fn card_rounding() -> CornerRadius {
    CornerRadius::same(0) // Sharp edges for tactical look
}

pub fn button_rounding() -> CornerRadius {
    CornerRadius::same(0) // Sharp edges
}

pub fn badge_rounding() -> CornerRadius {
    CornerRadius::same(2) // Slightly rounded to differentiate from structural cards
}

pub fn card_stroke() -> Stroke {
    Stroke::new(1.0, get_theme().border_subtle)
}

pub fn card_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(if get_theme().mode == ThemeMode::Dark { 140 } else { 20 }),
    }
}

pub fn hover_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: egui::Color32::from_black_alpha(if get_theme().mode == ThemeMode::Dark { 180 } else { 40 }),
    }
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

    visuals.selection.bg_fill = theme.accent.linear_multiply(0.4); 
    visuals.selection.stroke = Stroke::new(1.0, theme.accent);

    visuals.window_stroke = Stroke::new(2.0, theme.accent); // Strong window border
    visuals.window_corner_radius = CornerRadius::same(0); // Sharp windows
    visuals.window_shadow = card_shadow();

    visuals.striped = true;

    style.visuals = visuals;

    // Enhanced Spacing for modern breathable UI
    style.spacing.item_spacing = Vec2::new(16.0, 12.0); // More spacing between items
    style.spacing.window_margin = egui::Margin::same(24);
    style.spacing.button_padding = Vec2::new(20.0, 10.0); // Larger buttons
    style.spacing.interact_size = Vec2::new(56.0, 36.0);
    style.spacing.indent = 20.0;
    
    // Smooth animations
    style.animation_time = 0.15;

    ctx.set_style(style);
}
