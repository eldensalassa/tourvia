use egui::{self, Align, FontFamily, FontId, RichText, Stroke, Vec2};

use crate::app::TourviaApp;
use crate::domain::match_model::{Match, MatchStatus};
use crate::ui::theme;

pub fn render(app: &mut TourviaApp, ctx: &egui::Context) {
    ctx.request_repaint_after(std::time::Duration::from_secs(1));
    ensure_scoreboard_selection(app);

    egui::TopBottomPanel::top("scoreboard_top")
        .frame(
            egui::Frame::new()
                .fill(theme::BG_PANEL())
                .inner_margin(egui::Margin::symmetric(24, 16)),
        )
        .show(ctx, |ui| render_top_bar(app, ui));

    egui::TopBottomPanel::bottom("scoreboard_operator")
        .frame(
            egui::Frame::new()
                .fill(theme::BG_PANEL())
                .inner_margin(egui::Margin::symmetric(24, 14)),
        )
        .show(ctx, |ui| render_operator_bar(app, ui));

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(theme::BG_DARK())
                .inner_margin(egui::Margin::symmetric(32, 24)),
        )
        .show(ctx, |ui| {
            let current_match = current_match(app);
            if let Some(m) = current_match {
                render_scoreboard(app, ui, &m);
            } else {
                render_empty(ui);
            }
        });

    if app.show_match_modal {
        crate::ui::match_panel::render_modal(app, ctx);
    }
}

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

fn render_top_bar(app: &mut TourviaApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let input_fill = ui.visuals().extreme_bg_color;
        if ui
            .add(
                egui::Button::new(
                    RichText::new("Back")
                        .size(13.0)
                        .color(theme::TEXT_SECONDARY()),
                )
                .fill(input_fill),
            )
            .clicked()
        {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            app.close_scoreboard();
        }

        ui.add_space(12.0);

        if let Some(t) = &app.active_tournament {
            if let Some(texture) = app.tournament_logo_textures.get(&t.id) {
                ui.add(
                    egui::Image::new(texture)
                        .fit_to_exact_size(Vec2::new(36.0, 36.0))
                        .corner_radius(4),
                );
                ui.add_space(8.0);
            }
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(&t.name)
                        .size(18.0)
                        .color(theme::TEXT_PRIMARY())
                        .strong(),
                );
                let subtitle = if t.game_name.trim().is_empty() {
                    t.tournament_type.as_str().to_string()
                } else {
                    format!("{} / {}", t.game_name, t.tournament_type.as_str())
                };
                ui.label(
                    RichText::new(subtitle)
                        .size(12.0)
                        .color(theme::TEXT_MUTED()),
                );
            });
        }

        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            let display_label = if app.scoreboard_display_window_open {
                "Close Display"
            } else {
                "Open Display"
            };
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(display_label)
                            .size(12.0)
                            .color(theme::TEXT_SECONDARY()),
                    )
                    .fill(theme::BG_CARD()),
                )
                .clicked()
            {
                app.toggle_scoreboard_display_window();
            }

            ui.add_space(8.0);

            if ui
                .add(
                    egui::Button::new(
                        RichText::new("Fullscreen")
                            .size(12.0)
                            .color(theme::BG_DARK())
                            .strong(),
                    )
                    .fill(theme::ACCENT_BRONZE()),
                )
                .clicked()
            {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }

            ui.add_space(8.0);

            if ui
                .add(
                    egui::Button::new(
                        RichText::new("Open Match")
                            .size(12.0)
                            .color(theme::TEXT_SECONDARY()),
                    )
                    .fill(theme::BG_CARD()),
                )
                .clicked()
            {
                if app.selected_match.is_some() {
                    app.show_match_modal = true;
                }
            }
        });
    });
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

fn render_operator_bar(app: &mut TourviaApp, ui: &mut egui::Ui) {
    ensure_scoreboard_selection(app);
    let matches = selectable_matches(app);
    let current = current_match(app);

    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(
                    RichText::new("Previous")
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY()),
                )
                .fill(theme::BG_CARD()),
            )
            .clicked()
        {
            select_relative_match(app, -1);
        }

        if ui
            .add(
                egui::Button::new(
                    RichText::new("Next")
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY()),
                )
                .fill(theme::BG_CARD()),
            )
            .clicked()
        {
            select_relative_match(app, 1);
        }

        ui.add_space(12.0);
        ui.label(
            RichText::new(format!("{} match(es)", matches.len()))
                .size(12.0)
                .color(theme::TEXT_MUTED()),
        );

        if let Some(m) = current {
            ui.add_space(20.0);
            ui.label(
                RichText::new(format!(
                    "Displaying: {} vs {}",
                    player_name(&m.player1_name),
                    player_name(&m.player2_name)
                ))
                .size(13.0)
                .color(theme::TEXT_SECONDARY()),
            );

            match m.status {
                MatchStatus::Pending => {
                    let players_ready = m.player1_id.is_some() && m.player2_id.is_some();
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        let label = if players_ready {
                            "Start Match"
                        } else {
                            "Waiting For Opponents"
                        };
                        let button = egui::Button::new(
                            RichText::new(label)
                                .size(12.0)
                                .color(theme::BG_DARK())
                                .strong(),
                        )
                        .fill(if players_ready {
                            theme::SUCCESS()
                        } else {
                            theme::TEXT_MUTED()
                        });

                        if ui.add_enabled(players_ready, button).clicked() {
                            app.selected_match = Some(m.id.clone());
                            app.start_match();
                        }
                    });
                }
                MatchStatus::InProgress => {
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("End Match & Save")
                                        .size(12.0)
                                        .color(egui::Color32::WHITE)
                                        .strong(),
                                )
                                .fill(theme::ERROR().linear_multiply(0.85)),
                            )
                            .clicked()
                        {
                            app.selected_match = Some(m.id.clone());
                            app.end_match();
                        }

                        score_stepper(ui, app, &m.id, &m.player2_name, m.score2, 0, 1);
                        ui.add_space(10.0);
                        score_stepper(ui, app, &m.id, &m.player1_name, m.score1, 1, 0);
                    });
                }
                MatchStatus::Completed => {
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Final result saved.")
                                .size(12.0)
                                .color(theme::SUCCESS()),
                        );
                    });
                }
                MatchStatus::Bye => {
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Bye match.")
                                .size(12.0)
                                .color(theme::TEXT_MUTED()),
                        );
                    });
                }
            }
        }
    });
}

fn score_stepper(
    ui: &mut egui::Ui,
    app: &mut TourviaApp,
    match_id: &str,
    player: &str,
    score: i32,
    p1_delta: i32,
    p2_delta: i32,
) {
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(RichText::new("+").size(14.0).strong())
                    .fill(theme::BG_ELEVATED())
                    .min_size(Vec2::new(28.0, 28.0)),
            )
            .clicked()
        {
            app.selected_match = Some(match_id.to_owned());
            app.update_live_score(p1_delta, p2_delta);
        }
        ui.label(
            RichText::new(score.to_string())
                .size(16.0)
                .color(theme::TEXT_PRIMARY())
                .strong(),
        );
        if ui
            .add(
                egui::Button::new(RichText::new("-").size(14.0).strong())
                    .fill(theme::BG_ELEVATED())
                    .min_size(Vec2::new(28.0, 28.0)),
            )
            .clicked()
        {
            app.selected_match = Some(match_id.to_owned());
            app.update_live_score(-p1_delta, -p2_delta);
        }
        ui.label(
            RichText::new(player_name(player))
                .size(12.0)
                .color(theme::TEXT_MUTED()),
        );
    });
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
            RichText::new("Generate a bracket before opening scoreboard mode.")
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

fn select_relative_match(app: &mut TourviaApp, offset: isize) {
    let matches = selectable_matches(app);
    if matches.is_empty() {
        return;
    }

    let current_idx = app
        .selected_match
        .as_ref()
        .and_then(|id| matches.iter().position(|m| &m.id == id))
        .unwrap_or(0);

    let len = matches.len() as isize;
    let next_idx = (current_idx as isize + offset).rem_euclid(len) as usize;
    app.selected_match = Some(matches[next_idx].id.clone());
    app.score_input = [String::new(), String::new()];
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
