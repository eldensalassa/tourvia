use egui::{self, Ui, RichText, Vec2, Color32};

use crate::app::TourviaApp;
use crate::domain::tournament::TournamentType;
use crate::ui::theme;

/// Render the tournament creation form.
pub fn render(app: &mut TourviaApp, ui: &mut Ui) {
    ui.add_space(12.0);

    // Header with back button
    ui.horizontal(|ui| {
        if ui.add(
            egui::Button::new(RichText::new("← Back").color(theme::TEXT_SECONDARY).size(14.0))
                .fill(Color32::TRANSPARENT),
        ).clicked() {
            app.go_to_dashboard();
        }
        ui.add_space(8.0);
        ui.label(theme::heading_text("📝 Create New Tournament"));
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(20.0);

    let available_width = ui.available_width().max(300.0);
    let form_width = (available_width * 0.55).clamp(280.0, 520.0);

    ui.vertical_centered(|ui| {
        ui.set_max_width(form_width);

        egui::Frame::new()
            .fill(theme::BG_CARD)
            .stroke(theme::card_stroke())
            .corner_radius(theme::card_rounding())
            .inner_margin(egui::Margin::same(28))
            .show(ui, |ui| {
                ui.set_min_width(form_width - 56.0);

                // ─── Tournament Name ────────────────
                ui.label(theme::section_header("TOURNAMENT NAME"));
                ui.add_space(4.0);
                let name_edit = egui::TextEdit::singleline(&mut app.new_tournament_name)
                    .hint_text("e.g., Grand Championship 2026")
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::proportional(15.0))
                    .text_color(theme::TEXT_PRIMARY);
                let name_resp = ui.add(name_edit);
                if app.new_tournament_name.trim().is_empty() && name_resp.lost_focus() {
                    ui.label(RichText::new("Name is required").size(10.0).color(theme::ERROR));
                }

                ui.add_space(18.0);

                // ─── Game Name ──────────────────────
                ui.label(theme::section_header("GAME / DISCIPLINE"));
                ui.add_space(4.0);
                let game_edit = egui::TextEdit::singleline(&mut app.new_tournament_game)
                    .hint_text("e.g., Valorant, Mobile Legends, Chess")
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::proportional(14.0))
                    .text_color(theme::TEXT_PRIMARY);
                ui.add(game_edit);

                ui.add_space(18.0);

                // ─── Description ────────────────────
                ui.label(theme::section_header("DESCRIPTION (OPTIONAL)"));
                ui.add_space(4.0);
                let desc_edit = egui::TextEdit::multiline(&mut app.new_tournament_description)
                    .hint_text("Tournament description, rules, prize pool, etc.")
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .font(egui::FontId::proportional(13.0))
                    .text_color(theme::TEXT_PRIMARY);
                ui.add(desc_edit);

                ui.add_space(18.0);

                // ─── Tournament Type ────────────────
                ui.label(theme::section_header("TOURNAMENT FORMAT"));
                ui.add_space(6.0);

                ui.horizontal_wrapped(|ui| {
                    type_button(ui, app, TournamentType::SingleElimination, "⚡ Single Elimination", theme::ACCENT_BRONZE);
                    type_button(ui, app, TournamentType::DoubleElimination, "🔄 Double Elimination", theme::ACCENT_BRONZE_LIGHT);
                    type_button(ui, app, TournamentType::RoundRobin, "🔁 Round Robin", theme::ACCENT_BRONZE_DARK);
                });

                ui.add_space(6.0);
                // Type description
                let type_desc = match app.new_tournament_type {
                    TournamentType::SingleElimination => "Lose once and you're out. Fast & decisive.",
                    TournamentType::DoubleElimination => "Two chances — lose twice before elimination.",
                    TournamentType::RoundRobin => "Everyone plays everyone. Best overall record wins.",
                };
                ui.label(RichText::new(type_desc).size(11.0).color(theme::TEXT_MUTED).italics());

                ui.add_space(28.0);

                // ─── Create Button ──────────────────
                let can_create = !app.new_tournament_name.trim().is_empty();
                let create_btn = egui::Button::new(
                    RichText::new("🏆 Create Tournament")
                        .size(15.0)
                        .color(if can_create { theme::BG_DARK } else { theme::TEXT_MUTED })
                        .strong(),
                )
                .fill(if can_create { theme::ACCENT_BRONZE } else { theme::BG_CARD_HOVER })
                .corner_radius(theme::button_rounding())
                .min_size(Vec2::new(0.0, 42.0));

                if ui.add_enabled(can_create, create_btn).clicked() {
                    app.create_tournament();
                }

                // Status message
                if let Some((ref msg, ref msg_type)) = app.status_message {
                    ui.add_space(12.0);
                    let color = match msg_type {
                        crate::app::MessageType::Success => theme::SUCCESS,
                        crate::app::MessageType::Error => theme::ERROR,
                        crate::app::MessageType::Info => theme::INFO,
                    };
                    ui.label(RichText::new(msg).color(color).size(13.0));
                }
            });
    });
}

fn type_button(ui: &mut Ui, app: &mut TourviaApp, t_type: TournamentType, label: &str, color: Color32) {
    let is_selected = app.new_tournament_type == t_type;
    let btn = egui::Button::new(
        RichText::new(label)
            .size(12.0)
            .color(if is_selected { theme::BG_DARK } else { theme::TEXT_SECONDARY })
            .strong(),
    )
    .fill(if is_selected { color } else { theme::BG_CARD_HOVER })
    .corner_radius(theme::button_rounding())
    .min_size(Vec2::new(0.0, 32.0));

    if ui.add(btn).clicked() {
        app.new_tournament_type = t_type;
    }
}
