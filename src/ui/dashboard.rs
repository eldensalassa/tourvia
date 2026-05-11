use egui::{self, Ui, Color32, RichText, Stroke, Vec2};

use crate::app::TourviaApp;
use crate::ui::theme;

/// Render the main dashboard showing all saved tournaments.
pub fn render(app: &mut TourviaApp, ui: &mut Ui) {
    ui.add_space(12.0);

    // ─── Header ─────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(theme::heading_text("🏆 Tournament Dashboard"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let btn = egui::Button::new(
                RichText::new("+ New Tournament")
                    .color(theme::BG_DARK)
                    .strong()
                    .size(14.0),
            )
            .fill(theme::ACCENT_BRONZE)
            .corner_radius(theme::button_rounding())
            .min_size(Vec2::new(170.0, 36.0));

            if ui.add(btn).clicked() {
                app.show_tournament_form();
            }
        });
    });

    ui.add_space(8.0);

    // ─── Stats Overview ─────────────────────────────
    let total = app.tournaments.len();
    let active = app.tournaments.iter().filter(|t| t.status == crate::domain::tournament::TournamentStatus::InProgress).count();
    let completed = app.tournaments.iter().filter(|t| t.status == crate::domain::tournament::TournamentStatus::Completed).count();
    let draft = app.tournaments.iter().filter(|t| t.status == crate::domain::tournament::TournamentStatus::Draft).count();

    ui.horizontal(|ui| {
        stat_badge(ui, "📋 Total", total, theme::TEXT_PRIMARY);
        ui.add_space(4.0);
        stat_badge(ui, "📝 Draft", draft, theme::TEXT_MUTED);
        ui.add_space(4.0);
        stat_badge(ui, "▶ Active", active, theme::ACCENT_BRONZE);
        ui.add_space(4.0);
        stat_badge(ui, "✅ Done", completed, theme::SUCCESS);
    });

    ui.add_space(10.0);

    // ─── Search Bar ─────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(RichText::new("🔍").size(14.0));
        let search = egui::TextEdit::singleline(&mut app.search_query)
            .hint_text("Search tournaments...")
            .desired_width(300.0)
            .font(egui::FontId::proportional(13.0))
            .text_color(theme::TEXT_PRIMARY);
        ui.add(search);

        if !app.search_query.is_empty() {
            if ui.add(
                egui::Button::new(RichText::new("✕").size(12.0).color(theme::TEXT_MUTED))
                    .fill(Color32::TRANSPARENT)
            ).clicked() {
                app.search_query.clear();
            }
        }
    });

    ui.add_space(6.0);
    ui.add(egui::Separator::default().spacing(2.0));
    ui.add_space(8.0);

    // ─── Tournament List ────────────────────────────
    let filtered = app.filtered_tournaments();

    if app.tournaments.is_empty() {
        empty_state(ui);
        return;
    }

    if filtered.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("🔍").size(40.0));
            ui.add_space(8.0);
            ui.label(theme::label_text("No tournaments match your search."));
        });
        return;
    }

    ui.label(theme::small_text(&format!(
        "Showing {} of {} tournament(s)",
        filtered.len(),
        app.tournaments.len()
    )));
    ui.add_space(6.0);

    // Clone data needed for rendering to avoid borrow issues
    let filtered_data: Vec<(usize, crate::domain::tournament::Tournament)> =
        filtered.iter().map(|(idx, t)| (*idx, (*t).clone())).collect();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (idx, tournament) in &filtered_data {
            let idx = *idx;

            egui::Frame::new()
                .fill(theme::BG_CARD)
                .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
                .corner_radius(theme::card_rounding())
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Left section
                        ui.vertical(|ui| {
                            ui.set_min_width(320.0);

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&tournament.name)
                                        .size(16.0)
                                        .color(theme::TEXT_PRIMARY)
                                        .strong(),
                                );

                                // Status badge
                                let (status_color, status_text) = match tournament.status {
                                    crate::domain::tournament::TournamentStatus::Draft => {
                                        (theme::TEXT_MUTED, "📝 Draft")
                                    }
                                    crate::domain::tournament::TournamentStatus::InProgress => {
                                        (theme::ACCENT_BRONZE, "▶ In Progress")
                                    }
                                    crate::domain::tournament::TournamentStatus::Completed => {
                                        (theme::SUCCESS, "✅ Completed")
                                    }
                                };

                                ui.add(
                                    egui::Button::new(
                                        RichText::new(status_text).size(10.0).color(status_color),
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::new(1.0, status_color))
                                    .corner_radius(12),
                                );
                            });

                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
                                if !tournament.game_name.is_empty() {
                                    ui.label(
                                        RichText::new(&format!("🎮 {}", tournament.game_name))
                                            .size(11.0)
                                            .color(theme::ACCENT_BRONZE_LIGHT),
                                    );
                                    ui.label(RichText::new("•").size(11.0).color(theme::TEXT_MUTED));
                                }
                                ui.label(theme::small_text(&format!(
                                    "📋 {} • 👥 {} • 📅 {}",
                                    tournament.tournament_type.as_str(),
                                    tournament.participant_count,
                                    tournament.created_at
                                )));
                            });

                            if !tournament.description.is_empty() {
                                ui.add_space(2.0);
                                let desc = if tournament.description.len() > 80 {
                                    format!("{}...", &tournament.description[..80])
                                } else {
                                    tournament.description.clone()
                                };
                                ui.label(
                                    RichText::new(desc)
                                        .size(11.0)
                                        .color(theme::TEXT_MUTED)
                                        .italics(),
                                );
                            }
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Delete button with confirmation
                            if app.confirm_delete == Some(idx) {
                                if ui.add(
                                    egui::Button::new(
                                        RichText::new("Cancel").size(11.0).color(theme::TEXT_SECONDARY),
                                    ).fill(theme::BG_CARD_HOVER)
                                    .corner_radius(theme::button_rounding()),
                                ).clicked() {
                                    app.confirm_delete = None;
                                }

                                if ui.add(
                                    egui::Button::new(
                                        RichText::new("🗑 Confirm Delete").size(11.0).color(Color32::WHITE),
                                    ).fill(theme::ERROR)
                                    .corner_radius(theme::button_rounding()),
                                ).clicked() {
                                    app.delete_tournament_at(idx);
                                }
                            } else {
                                if ui.add(
                                    egui::Button::new(RichText::new("🗑").size(15.0))
                                        .fill(Color32::TRANSPARENT)
                                        .stroke(Stroke::new(1.0, theme::ERROR)),
                                ).on_hover_text("Delete Tournament").clicked() {
                                    app.confirm_delete = Some(idx);
                                }
                            }

                            // Open button
                            let open_btn = egui::Button::new(
                                RichText::new("Open")
                                    .size(13.0)
                                    .color(theme::ACCENT_BRONZE)
                                    .strong(),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, theme::ACCENT_BRONZE))
                            .corner_radius(theme::button_rounding());

                            if ui.add(open_btn).clicked() {
                                app.open_tournament(idx);
                            }
                        });
                    });
                });

            ui.add_space(6.0);
        }
    });
}

fn stat_badge(ui: &mut Ui, label: &str, count: usize, color: Color32) {
    egui::Frame::new()
        .fill(theme::BG_CARD)
        .stroke(Stroke::new(0.5, theme::BORDER_SUBTLE))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(14, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).size(11.0).color(theme::TEXT_MUTED));
                ui.label(RichText::new(count.to_string()).size(16.0).color(color).strong());
            });
        });
}

fn empty_state(ui: &mut Ui) {
    ui.add_space(80.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new("🎮").size(72.0));
        ui.add_space(16.0);
        ui.label(theme::subheading_text("No Tournaments Yet"));
        ui.add_space(8.0);
        ui.label(theme::label_text("Create your first tournament to get started!"));
        ui.add_space(4.0);
        ui.label(theme::small_text("Click the '+ New Tournament' button above"));
    });
}
