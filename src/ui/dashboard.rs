use egui::{self, Color32, RichText, Stroke, Ui, Vec2};

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
                    ui.horizontal(|ui| {
                        // Add application logo next to the title
                        ui.add(egui::Image::new(egui::include_image!("../assets/logo.png")).max_height(36.0));
                        
                        ui.label(
                            RichText::new("Tourvia")
                                .font(egui::FontId::proportional(32.0))
                                .color(theme::ACCENT_BRONZE())
                                .strong(),
                        );
                    });
                    ui.add_space(4.0);
                    ui.label(theme::body_text(
                        "Tournament Visualization & Administration",
                    ));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Theme toggle
                    let current_mode = theme::get_theme().mode;
                    let icon = if current_mode == theme::ThemeMode::Dark {
                        "✨ Light"
                    } else {
                        "🌙 Dark"
                    };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(icon).color(theme::TEXT_PRIMARY()))
                                .fill(theme::BG_ELEVATED())
                                .corner_radius(theme::button_rounding())
                                .min_size(Vec2::new(80.0, 36.0)),
                        )
                        .clicked()
                    {
                        let new_theme = if current_mode == theme::ThemeMode::Dark {
                            theme::ThemeConfig::light()
                        } else {
                            theme::ThemeConfig::dark()
                        };
                        theme::set_theme(new_theme);
                        app.theme_applied = false;
                    }

                    ui.add_space(8.0);

                    let import_btn = egui::Button::new(
                        RichText::new("📂 Import")
                            .color(theme::TEXT_PRIMARY())
                            .strong(),
                    )
                    .fill(theme::BG_CARD_HOVER())
                    .corner_radius(theme::button_rounding())
                    .min_size(Vec2::new(100.0, 36.0));
                    if ui.add(import_btn).clicked() {
                        app.import_json();
                    }

                    ui.add_space(8.0);

                    let roster_btn = egui::Button::new(
                        RichText::new("👥 Database")
                            .color(theme::TEXT_PRIMARY())
                            .strong(),
                    )
                    .fill(theme::BG_CARD_HOVER())
                    .corner_radius(theme::button_rounding())
                    .min_size(Vec2::new(140.0, 36.0));
                    if ui.add(roster_btn).clicked() {
                        app.current_view = crate::app::View::GlobalRoster;
                        app.load_rosters();
                    }

                    ui.add_space(8.0);

                    let new_btn = egui::Button::new(
                        RichText::new("+ New Tournament")
                            .color(theme::BG_DARK())
                            .strong(),
                    )
                    .fill(theme::ACCENT_BRONZE())
                    .corner_radius(theme::button_rounding())
                    .min_size(Vec2::new(160.0, 36.0));
                    if ui.add(new_btn).clicked() {
                        app.show_tournament_form();
                    }
                });
            });
        });

    ui.add_space(16.0);

    // ─── Stats Overview ─────────────────────────────
    let total = app.tournaments.len();
    let active = app
        .tournaments
        .iter()
        .filter(|t| t.status == crate::domain::tournament::TournamentStatus::InProgress)
        .count();
    let completed = app
        .tournaments
        .iter()
        .filter(|t| t.status == crate::domain::tournament::TournamentStatus::Completed)
        .count();
    let draft = app
        .tournaments
        .iter()
        .filter(|t| t.status == crate::domain::tournament::TournamentStatus::Draft)
        .count();

    ui.columns(4, |cols| {
        stat_card(&mut cols[0], "📋 Total Tournaments", total, theme::TEXT_PRIMARY());
        stat_card(&mut cols[1], "▶ Active", active, theme::ACCENT_BRONZE());
        stat_card(&mut cols[2], "✅ Completed", completed, theme::SUCCESS());
        stat_card(&mut cols[3], "📝 Drafts", draft, theme::TEXT_MUTED());
    });

    ui.add_space(16.0);

    let filtered_len = app.filtered_tournaments().len();
    let total_len = app.tournaments.len();

    // ─── Search Bar & List Header ─────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(RichText::new("🔍").size(14.0));
        let search = egui::TextEdit::singleline(&mut app.search_query)
            .hint_text("Search tournaments...")
            .desired_width(320.0)
            .font(egui::FontId::proportional(14.0))
            .text_color(theme::TEXT_PRIMARY());
        ui.add(search);

        if !app.search_query.is_empty() {
            if ui
                .add(
                    egui::Button::new(RichText::new("✕").size(12.0).color(theme::TEXT_MUTED()))
                        .fill(Color32::TRANSPARENT),
                )
                .clicked()
            {
                app.search_query.clear();
            }
        }
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(theme::small_text(&format!(
                "Showing {} of {} tournament(s)",
                filtered_len,
                total_len
            )));
        });
    });

    ui.add_space(8.0);
    ui.add(egui::Separator::default().spacing(0.0));
    ui.add_space(16.0);

    // ─── Tournament List ────────────────────────────
    if app.tournaments.is_empty() {
        empty_state(ui);
        return;
    }

    let filtered = app.filtered_tournaments();

    if filtered.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("🔍").size(40.0));
            ui.add_space(8.0);
            ui.label(theme::label_text("No tournaments match your search."));
        });
        return;
    }

    // Clone data needed for rendering to avoid borrow issues
    let filtered_data: Vec<(usize, crate::domain::tournament::Tournament)> = filtered
        .iter()
        .map(|(idx, t)| (*idx, (*t).clone()))
        .collect();

    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        app.ensure_tournament_logos_loaded(ui.ctx());

        let card_width = 340.0;
        let spacing = 16.0;

        egui::Frame::NONE
            .inner_margin(egui::Margin { left: 8, right: 24, top: 8, bottom: 16 }) // Adds extra right margin to push away from scrollbar
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(spacing, spacing);
                
                // Use left_to_right with Align::Min to force top-alignment and prevent staircase effect
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true), |ui| {
                    for (idx, tournament) in &filtered_data {
                        let idx = *idx;

                        let card_size = Vec2::new(card_width, 160.0);
                        ui.allocate_ui(card_size, |ui| {
                            egui::Frame::new()
                                .fill(theme::BG_CARD())
                                .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE()))
                                .corner_radius(theme::card_rounding())
                                .inner_margin(egui::Margin::same(16))
                                .show(ui, |ui| {
                                    ui.set_min_size(Vec2::new(card_width - 32.0, 160.0 - 32.0));
                                    ui.set_max_size(Vec2::new(card_width - 32.0, 160.0 - 32.0));

                                    ui.vertical(|ui| {
                                        // Top row: Logo + Details
                                        ui.horizontal(|ui| {
                                            // Logo
                                            if let Some(texture) = app.tournament_logo_textures.get(&tournament.id) {
                                                ui.add(
                                                    egui::Image::new(texture)
                                                        .fit_to_exact_size(Vec2::new(64.0, 64.0))
                                                        .corner_radius(8),
                                                );
                                            } else {
                                                // Placeholder
                                                let (rect, _resp) = ui.allocate_exact_size(
                                                    Vec2::new(64.0, 64.0),
                                                    egui::Sense::hover(),
                                                );
                                                ui.painter().rect_filled(rect, 8.0, theme::BG_ELEVATED());
                                                ui.painter().text(
                                                    rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    "🏆",
                                                    egui::FontId::proportional(32.0),
                                                    theme::TEXT_MUTED(),
                                                );
                                            }

                                            ui.add_space(12.0);

                                            ui.vertical(|ui| {
                                                let name = if tournament.name.len() > 25 {
                                                    format!("{}...", &tournament.name[..22])
                                                } else {
                                                    tournament.name.clone()
                                                };
                                                ui.label(
                                                    RichText::new(name)
                                                        .size(16.0)
                                                        .color(theme::TEXT_PRIMARY())
                                                        .strong(),
                                                );

                                                ui.add_space(2.0);

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
                                                        RichText::new(status_text)
                                                            .size(10.0)
                                                            .color(status_color),
                                                    )
                                                    .fill(Color32::TRANSPARENT)
                                                    .stroke(Stroke::new(1.0, status_color))
                                                    .corner_radius(12)
                                                    .min_size(Vec2::new(0.0, 20.0)),
                                                );
                                            });
                                        });

                                        ui.add_space(12.0);

                                        // Mid row: Game & Metadata
                                        let date = tournament.created_at.split(' ').next().unwrap_or(&tournament.created_at);
                                        ui.label(
                                            RichText::new(format!("🎮 {}", tournament.game_name))
                                                .size(13.0)
                                                .color(theme::ACCENT_BRONZE_LIGHT())
                                                .strong()
                                        );
                                        ui.add_space(2.0);
                                        ui.label(theme::small_text(&format!(
                                            "📋 {} • 👥 {} • 📅 {}",
                                            tournament.tournament_type.as_str(),
                                            tournament.participant_count,
                                            date
                                        )));

                                        // Fill remaining space
                                        let remaining = ui.available_height();
                                        if remaining > 32.0 {
                                            ui.add_space(remaining - 32.0);
                                        }

                                        // Bottom row: Actions
                                        ui.horizontal(|ui| {
                                            let open_btn = egui::Button::new(
                                                RichText::new("Open Tournament")
                                                    .size(13.0)
                                                    .color(theme::BG_DARK())
                                                    .strong(),
                                            )
                                            .fill(theme::ACCENT_BRONZE())
                                            .corner_radius(theme::button_rounding());

                                            if ui.add_sized(Vec2::new(card_width - 86.0, 32.0), open_btn).clicked() {
                                                app.open_tournament(idx);
                                            }

                                            // Delete
                                            if app.confirm_delete == Some(idx) {
                                                if ui
                                                    .add_sized(Vec2::new(32.0, 32.0), egui::Button::new(RichText::new("✓").color(theme::BG_DARK())).fill(theme::ERROR()).corner_radius(theme::button_rounding()))
                                                    .on_hover_text("Confirm Delete")
                                                    .clicked()
                                                {
                                                    app.delete_tournament_at(idx);
                                                    app.confirm_delete = None;
                                                }
                                            } else {
                                                if ui
                                                    .add_sized(
                                                        Vec2::new(32.0, 32.0),
                                                        egui::Button::new(RichText::new("🗑").color(theme::ERROR()))
                                                            .fill(theme::BG_CARD_HOVER())
                                                            .corner_radius(theme::button_rounding())
                                                    )
                                                    .on_hover_text("Delete Tournament")
                                                    .clicked()
                                                {
                                                    app.confirm_delete = Some(idx);
                                                }
                                            }
                                        });
                                    });
                                });
                        });
                    }
                });
            });
    });
}

fn stat_card(ui: &mut Ui, label: &str, count: usize, color: Color32) {
    egui::Frame::NONE
        .fill(theme::BG_CARD())
        .corner_radius(theme::card_rounding())
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE()))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new(label).color(theme::TEXT_MUTED()).size(14.0));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(count.to_string())
                        .color(color)
                        .size(32.0)
                        .strong(),
                );
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
        ui.label(theme::label_text(
            "Create your first tournament to get started!",
        ));
        ui.add_space(4.0);
        ui.label(theme::small_text(
            "Click the '+ New Tournament' button above",
        ));
    });
}
