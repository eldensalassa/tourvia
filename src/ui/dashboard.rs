use egui::{self, Ui, Color32, RichText, Stroke, Vec2};

use crate::app::TourviaApp;
use crate::ui::theme;

/// Render the main dashboard showing all saved tournaments.
pub fn render(app: &mut TourviaApp, ui: &mut Ui) {
    ui.add_space(12.0);

    // ─── Header Hero Section ─────────────────────────────
    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .corner_radius(theme::card_rounding())
        .stroke(theme::card_stroke())
        .inner_margin(egui::Margin::same(24))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Tourvia").font(egui::FontId::proportional(32.0)).color(theme::ACCENT_BRONZE()).strong());
                    ui.label(theme::body_text("Tournament Visualization & Administration"));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Theme toggle
                    let current_mode = theme::get_theme().mode;
                    let icon = if current_mode == theme::ThemeMode::Dark { "✨ Light" } else { "🌙 Dark" };
                    if ui.add(egui::Button::new(RichText::new(icon).color(theme::TEXT_PRIMARY())).fill(theme::BG_ELEVATED()).corner_radius(theme::button_rounding()).min_size(Vec2::new(80.0, 36.0))).clicked() {
                        let new_theme = if current_mode == theme::ThemeMode::Dark { theme::ThemeConfig::light() } else { theme::ThemeConfig::dark() };
                        theme::set_theme(new_theme);
                        app.theme_applied = false; 
                    }

                    ui.add_space(8.0);

                    let import_btn = egui::Button::new(RichText::new("📂 Import").color(theme::TEXT_PRIMARY()).strong()).fill(theme::BG_CARD_HOVER()).corner_radius(theme::button_rounding()).min_size(Vec2::new(100.0, 36.0));
                    if ui.add(import_btn).clicked() { app.import_json(); }

                    ui.add_space(8.0);

                    let roster_btn = egui::Button::new(RichText::new("👥 Database").color(theme::TEXT_PRIMARY()).strong()).fill(theme::BG_CARD_HOVER()).corner_radius(theme::button_rounding()).min_size(Vec2::new(140.0, 36.0));
                    if ui.add(roster_btn).clicked() {
                        app.current_view = crate::app::View::GlobalRoster;
                        app.load_rosters();
                    }

                    ui.add_space(8.0);

                    let new_btn = egui::Button::new(RichText::new("+ New Tournament").color(theme::BG_DARK()).strong()).fill(theme::ACCENT_BRONZE()).corner_radius(theme::button_rounding()).min_size(Vec2::new(140.0, 36.0));
                    if ui.add(new_btn).clicked() { app.show_tournament_form(); }
                });
            });
        });

    ui.add_space(16.0);

    // ─── Stats Overview ─────────────────────────────
    let total = app.tournaments.len();
    let active = app.tournaments.iter().filter(|t| t.status == crate::domain::tournament::TournamentStatus::InProgress).count();
    let completed = app.tournaments.iter().filter(|t| t.status == crate::domain::tournament::TournamentStatus::Completed).count();
    let draft = app.tournaments.iter().filter(|t| t.status == crate::domain::tournament::TournamentStatus::Draft).count();

    ui.horizontal(|ui| {
        stat_card(ui, "📋 Total Tournaments", total, theme::TEXT_PRIMARY());
        ui.add_space(8.0);
        stat_card(ui, "▶ Active", active, theme::ACCENT_BRONZE());
        ui.add_space(8.0);
        stat_card(ui, "✅ Completed", completed, theme::SUCCESS());
        ui.add_space(8.0);
        stat_card(ui, "📝 Drafts", draft, theme::TEXT_MUTED());
    });

    ui.add_space(10.0);

    // ─── Search Bar ─────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(RichText::new("🔍").size(14.0));
        let search = egui::TextEdit::singleline(&mut app.search_query)
            .hint_text("Search tournaments...")
            .desired_width(300.0)
            .font(egui::FontId::proportional(13.0))
            .text_color(theme::TEXT_PRIMARY());
        ui.add(search);

        if !app.search_query.is_empty() {
            if ui.add(
                egui::Button::new(RichText::new("✕").size(12.0).color(theme::TEXT_MUTED()))
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
                .fill(theme::BG_CARD())
                .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE()))
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
                                        .color(theme::TEXT_PRIMARY())
                                        .strong(),
                                );

                                // Status badge
                                let (status_color, status_text) = match tournament.status {
                                    crate::domain::tournament::TournamentStatus::Draft => {
                                        (theme::TEXT_MUTED(), "📝 Draft")
                                    }
                                    crate::domain::tournament::TournamentStatus::InProgress => {
                                        (theme::ACCENT_BRONZE(), "▶ In Progress")
                                    }
                                    crate::domain::tournament::TournamentStatus::Completed => {
                                        (theme::SUCCESS(), "✅ Completed")
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
                                            .color(theme::ACCENT_BRONZE_LIGHT()),
                                    );
                                    ui.label(RichText::new("•").size(11.0).color(theme::TEXT_MUTED()));
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
                                        .color(theme::TEXT_MUTED())
                                        .italics(),
                                );
                            }
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Delete button with confirmation
                            if app.confirm_delete == Some(idx) {
                                if ui.add(
                                    egui::Button::new(
                                        RichText::new("Cancel").size(11.0).color(theme::TEXT_SECONDARY()),
                                    ).fill(theme::BG_CARD_HOVER())
                                    .corner_radius(theme::button_rounding()),
                                ).clicked() {
                                    app.confirm_delete = None;
                                }

                                if ui.add(
                                    egui::Button::new(
                                        RichText::new("🗑 Confirm Delete").size(11.0).color(Color32::WHITE),
                                    ).fill(theme::ERROR())
                                    .corner_radius(theme::button_rounding()),
                                ).clicked() {
                                    app.delete_tournament_at(idx);
                                }
                            } else {
                                if ui.add(
                                    egui::Button::new(RichText::new("🗑").size(15.0))
                                        .fill(Color32::TRANSPARENT)
                                        .stroke(Stroke::new(1.0, theme::ERROR())),
                                ).on_hover_text("Delete Tournament").clicked() {
                                    app.confirm_delete = Some(idx);
                                }
                            }

                            // Open button
                            let open_btn = egui::Button::new(
                                RichText::new("Open")
                                    .size(13.0)
                                    .color(theme::ACCENT_BRONZE())
                                    .strong(),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, theme::ACCENT_BRONZE()))
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

fn stat_card(ui: &mut Ui, label: &str, count: usize, color: Color32) {
    egui::Frame::NONE
        .fill(theme::BG_CARD())
        .corner_radius(theme::card_rounding())
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE()))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(180.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(label).color(theme::TEXT_MUTED()).size(14.0));
                ui.add_space(4.0);
                ui.label(RichText::new(count.to_string()).color(color).size(32.0).strong());
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
