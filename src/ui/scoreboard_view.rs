use egui::{self, Align, FontFamily, FontId, RichText, Stroke, Vec2};

use crate::app::TourviaApp;
use crate::domain::match_model::{Match, MatchStatus};
use crate::ui::theme;

pub fn render_display_window(app: &mut TourviaApp, ctx: &egui::Context) {
    ctx.request_repaint_after(std::time::Duration::from_secs(1));
    ensure_scoreboard_selection(app);

    let current_match = current_match(app);
    let builder = egui::ViewportBuilder::default()
        .with_title("Tourvia Scoreboard Display")
        .with_inner_size([1280.0, 720.0])
        .with_min_inner_size([900.0, 520.0])
        .with_resizable(true);

    let mut close_requested = false;
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("tourvia_scoreboard_display"),
        builder,
        |viewport_ctx, _class| {
            viewport_ctx.request_repaint_after(std::time::Duration::from_secs(1));
            if viewport_ctx.input(|i| i.viewport().close_requested()) {
                close_requested = true;
            }

            egui::CentralPanel::default()
                .frame(
                    egui::Frame::new()
                        .fill(theme::BG_DARK())
                        .inner_margin(egui::Margin::symmetric(40, 28)),
                )
                .show(viewport_ctx, |ui| {
                    if let Some(m) = &current_match {
                        render_scoreboard(app, ui, m);
                    } else {
                        render_empty(ui);
                    }
                });
        },
    );

    if close_requested {
        app.scoreboard_display_window_open = false;
    }
}

fn render_scoreboard(app: &TourviaApp, ui: &mut egui::Ui, m: &Match) {
    let round_name = round_name(app, &m.round_id);
    let (status_text, status_color) = status_info(&m.status);

    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(status_text)
                .size(18.0)
                .color(status_color)
                .strong(),
        );
        ui.label(
            RichText::new(round_name)
                .size(15.0)
                .color(theme::TEXT_MUTED()),
        );
    });

    ui.add_space(28.0);

    let available_height = (ui.available_height() - 126.0).max(280.0);
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), available_height),
        egui::Layout::top_down(Align::Center),
        |ui| {
            ui.columns(3, |cols| {
                render_team_panel(app, &mut cols[0], m, 1);
                render_center_score(&mut cols[1], m);
                render_team_panel(app, &mut cols[2], m, 2);
            });
        },
    );

    ui.add_space(18.0);
    render_next_match(app, ui, m);
}

fn render_team_panel(app: &TourviaApp, ui: &mut egui::Ui, m: &Match, slot: i32) {
    let (id, name, score, is_winner) = if slot == 1 {
        (
            m.player1_id.as_deref(),
            player_name(&m.player1_name),
            m.score1,
            m.winner_id.is_some() && m.player1_id == m.winner_id,
        )
    } else {
        (
            m.player2_id.as_deref(),
            player_name(&m.player2_name),
            m.score2,
            m.winner_id.is_some() && m.player2_id == m.winner_id,
        )
    };

    let stroke_color = if is_winner {
        theme::SUCCESS()
    } else {
        theme::BORDER_SUBTLE()
    };
    let bg = if is_winner {
        theme::SUCCESS().linear_multiply(0.12)
    } else {
        theme::BG_PANEL()
    };

    egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0, stroke_color))
        .corner_radius(8)
        .inner_margin(egui::Margin::same(28))
        .show(ui, |ui| {
            ui.set_min_height(320.0);
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);
                if let Some(id) = id {
                    if let Some(texture) = app.logo_textures.get(id) {
                        ui.add(
                            egui::Image::new(texture)
                                .fit_to_exact_size(Vec2::new(132.0, 132.0))
                                .corner_radius(8),
                        );
                    } else {
                        render_logo_placeholder(ui);
                    }
                } else {
                    render_logo_placeholder(ui);
                }

                ui.add_space(28.0);
                ui.label(
                    RichText::new(name)
                        .size(34.0)
                        .color(theme::TEXT_PRIMARY())
                        .strong(),
                );

                if m.status == MatchStatus::Completed || m.status == MatchStatus::InProgress {
                    ui.add_space(20.0);
                    ui.label(
                        RichText::new(score.to_string())
                            .font(FontId::new(72.0, FontFamily::Proportional))
                            .color(if is_winner {
                                theme::SUCCESS()
                            } else {
                                theme::TEXT_PRIMARY()
                            })
                            .strong(),
                    );
                }

                if is_winner {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Winner")
                            .size(14.0)
                            .color(theme::SUCCESS())
                            .strong(),
                    );
                }
            });
        });
}

fn render_center_score(ui: &mut egui::Ui, m: &Match) {
    ui.vertical_centered(|ui| {
        ui.add_space(88.0);
        let label = match m.status {
            MatchStatus::Completed => "Final",
            MatchStatus::InProgress => "Live",
            MatchStatus::Bye => "Bye",
            MatchStatus::Pending => "VS",
        };
        ui.label(
            RichText::new(label)
                .size(24.0)
                .color(theme::TEXT_MUTED())
                .strong(),
        );
        ui.add_space(20.0);

        if m.status == MatchStatus::Completed || m.status == MatchStatus::InProgress {
            ui.label(
                RichText::new(format!("{} : {}", m.score1, m.score2))
                    .font(FontId::new(54.0, FontFamily::Proportional))
                    .color(theme::ACCENT_BRONZE())
                    .strong(),
            );
        } else {
            ui.label(
                RichText::new("VS")
                    .font(FontId::new(64.0, FontFamily::Proportional))
                    .color(theme::ACCENT_BRONZE())
                    .strong(),
            );
        }
    });
}

fn render_next_match(app: &TourviaApp, ui: &mut egui::Ui, current: &Match) {
    if let Some(next) = next_match(app, current) {
        egui::Frame::new()
            .fill(theme::BG_PANEL())
            .stroke(theme::card_stroke())
            .corner_radius(8)
            .inner_margin(egui::Margin::symmetric(18, 12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Next Up")
                            .size(13.0)
                            .color(theme::ACCENT_BRONZE_LIGHT())
                            .strong(),
                    );
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new(round_name(app, &next.round_id))
                            .size(13.0)
                            .color(theme::TEXT_MUTED()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} vs {}",
                                player_name(&next.player1_name),
                                player_name(&next.player2_name)
                            ))
                            .size(16.0)
                            .color(theme::TEXT_PRIMARY())
                            .strong(),
                        );
                    });
                });
            });
    }
}

fn render_logo_placeholder(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(132.0, 132.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 8.0, theme::BG_CARD());
    ui.painter().rect_stroke(
        rect,
        8.0,
        Stroke::new(1.0, theme::BORDER_SUBTLE()),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "TBD",
        FontId::new(22.0, FontFamily::Proportional),
        theme::TEXT_MUTED(),
    );
}

fn render_empty(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(180.0);
        ui.label(
            RichText::new("No match available")
                .size(28.0)
                .color(theme::TEXT_PRIMARY())
                .strong(),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new("Generate a bracket before opening the display.")
                .size(15.0)
                .color(theme::TEXT_MUTED()),
        );
    });
}

fn ensure_scoreboard_selection(app: &mut TourviaApp) {
    let matches = selectable_matches(app);
    if matches.is_empty() {
        app.selected_match = None;
        return;
    }

    let selected_exists = app
        .selected_match
        .as_ref()
        .map(|id| matches.iter().any(|m| &m.id == id))
        .unwrap_or(false);

    if selected_exists {
        return;
    }

    let fallback_id = matches
        .iter()
        .find(|m| m.status == MatchStatus::InProgress)
        .or_else(|| {
            matches.iter().find(|m| {
                m.status == MatchStatus::Pending
                    && !m.player1_name.is_empty()
                    && !m.player2_name.is_empty()
            })
        })
        .or_else(|| matches.iter().find(|m| m.status == MatchStatus::Pending))
        .or_else(|| {
            matches
                .iter()
                .rev()
                .find(|m| m.status == MatchStatus::Completed)
        })
        .or_else(|| matches.first())
        .map(|m| m.id.clone());

    if app.selected_match != fallback_id {
        app.score_input = [String::new(), String::new()];
    }
    app.selected_match = fallback_id;
}

fn current_match(app: &TourviaApp) -> Option<Match> {
    let selected = app.selected_match.as_ref()?;
    app.matches.iter().find(|m| &m.id == selected).cloned()
}

fn selectable_matches(app: &TourviaApp) -> Vec<Match> {
    app.matches
        .iter()
        .filter(|m| m.status != MatchStatus::Bye)
        .cloned()
        .collect()
}

fn next_match(app: &TourviaApp, current: &Match) -> Option<Match> {
    let matches = selectable_matches(app);
    let current_idx = matches.iter().position(|m| m.id == current.id)?;

    matches
        .iter()
        .skip(current_idx + 1)
        .find(|m| m.status == MatchStatus::InProgress || m.status == MatchStatus::Pending)
        .cloned()
        .or_else(|| {
            matches
                .iter()
                .take(current_idx)
                .find(|m| m.status == MatchStatus::InProgress || m.status == MatchStatus::Pending)
                .cloned()
        })
}

fn round_name(app: &TourviaApp, round_id: &str) -> String {
    app.rounds
        .iter()
        .find(|r| r.id == round_id)
        .map(|r| r.name.clone())
        .unwrap_or_else(|| "Match".to_string())
}

fn player_name(name: &str) -> &str {
    if name.trim().is_empty() {
        "TBD"
    } else {
        name
    }
}

fn status_info(status: &MatchStatus) -> (&'static str, egui::Color32) {
    match status {
        MatchStatus::Pending => ("Pending", theme::TEXT_MUTED()),
        MatchStatus::InProgress => ("Live Match", theme::ACCENT_BRONZE()),
        MatchStatus::Completed => ("Final Result", theme::SUCCESS()),
        MatchStatus::Bye => ("Bye", theme::WARNING()),
    }
}
