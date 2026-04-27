use egui::{self, Ui, Pos2, Rect, Vec2, Stroke, StrokeKind, RichText, Sense, FontId, Align2, FontFamily, CornerRadius};

use crate::app::TourviaApp;
use crate::domain::match_model::MatchStatus;
use crate::domain::tournament::TournamentType;
use crate::ui::theme;

const MATCH_CARD_WIDTH: f32 = 200.0;
const MATCH_CARD_HEIGHT: f32 = 60.0;   // Stacked: 2 rows of 30
const ROUND_HORIZONTAL_GAP: f32 = 60.0;
const MATCH_VERTICAL_GAP: f32 = 30.0;
const ROUND_HEADER_HEIGHT: f32 = 40.0;

pub fn render(app: &mut TourviaApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(theme::subheading_text("Bracket"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new(RichText::new("+").size(14.0).color(theme::TEXT_SECONDARY)).fill(theme::BG_CARD).min_size(Vec2::new(28.0, 24.0))).clicked() {
                app.bracket_zoom = (app.bracket_zoom + 0.1).min(2.0);
            }
            ui.label(RichText::new(&format!("{}%", (app.bracket_zoom * 100.0) as i32)).size(11.0).color(theme::TEXT_MUTED));
            if ui.add(egui::Button::new(RichText::new("−").size(14.0).color(theme::TEXT_SECONDARY)).fill(theme::BG_CARD).min_size(Vec2::new(28.0, 24.0))).clicked() {
                app.bracket_zoom = (app.bracket_zoom - 0.1).max(0.4);
            }
            ui.label(RichText::new("🔍").size(12.0));
        });
    });
    ui.add_space(8.0);

    if app.rounds.is_empty() || app.matches.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.label(theme::subheading_text("No Bracket Generated"));
            ui.label(theme::label_text("Go to Participants tab to add players and generate the bracket."));
        });
        return;
    }

    if let Some(ref champion) = app.champion_name {
        egui::Frame::new()
            .fill(egui::Color32::from_rgba_premultiplied(205, 127, 50, 20))
            .stroke(Stroke::new(1.0, theme::ACCENT_BRONZE))
            .corner_radius(6)
            .inner_margin(egui::Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🏆").size(20.0));
                    ui.label(theme::champion_text(&format!("Champion: {}", champion)));
                });
            });
        ui.add_space(16.0);
    }

    if let Some(ref t) = app.active_tournament {
        if t.tournament_type == TournamentType::RoundRobin {
            render_round_robin_view(app, ui);
            return;
        }
    }

    render_elimination_bracket(app, ui);
}

fn render_round_robin_view(app: &mut TourviaApp, ui: &mut Ui) {
    let mut clicked_match_id = None;

    for round in &app.rounds.clone() {
        ui.add_space(8.0);
        egui::Frame::new()
            .fill(theme::BG_PANEL)
            .stroke(theme::card_stroke())
            .corner_radius(theme::card_rounding())
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.label(theme::section_header(&round.name));
                ui.add_space(8.0);

                let round_matches: Vec<_> = app.matches.iter().filter(|m| m.round_id == round.id).collect();

                for m in &round_matches {
                    let is_selected = app.selected_match.as_ref() == Some(&m.id);
                    let bg = if is_selected { theme::BG_ELEVATED } else { theme::BG_CARD };
                    let border = if is_selected { theme::ACCENT_BRONZE } else { theme::BORDER_SUBTLE };

                    let resp = egui::Frame::new()
                        .fill(bg)
                        .stroke(Stroke::new(if is_selected { 1.5 } else { 0.5 }, border))
                        .corner_radius(4)
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let p1 = if m.player1_name.is_empty() { "TBD" } else { &m.player1_name };
                                let p1_color = if m.winner_id.is_some() && m.player1_id == m.winner_id { theme::SUCCESS } else { theme::TEXT_PRIMARY };
                                ui.allocate_ui_with_layout(Vec2::new(150.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(p1).size(14.0).color(p1_color).strong());
                                    if m.status == MatchStatus::Completed {
                                        ui.add_space(8.0);
                                        ui.label(RichText::new(m.score1.to_string()).size(14.0).color(p1_color).strong());
                                    }
                                });

                                ui.allocate_ui_with_layout(Vec2::new(40.0, 20.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                    ui.label(RichText::new("vs").size(12.0).color(theme::TEXT_MUTED));
                                });

                                let p2 = if m.player2_name.is_empty() { "TBD" } else { &m.player2_name };
                                let p2_color = if m.winner_id.is_some() && m.player2_id == m.winner_id { theme::SUCCESS } else { theme::TEXT_PRIMARY };
                                ui.allocate_ui_with_layout(Vec2::new(150.0, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    if m.status == MatchStatus::Completed {
                                        ui.label(RichText::new(m.score2.to_string()).size(14.0).color(p2_color).strong());
                                        ui.add_space(8.0);
                                    }
                                    ui.label(RichText::new(p2).size(14.0).color(p2_color).strong());
                                });

                                let (sc, st) = match m.status {
                                    MatchStatus::Completed => (theme::SUCCESS, "✅"),
                                    MatchStatus::InProgress => (theme::ACCENT_BRONZE, "▶"),
                                    MatchStatus::Bye => (theme::WARNING, "BYE"),
                                    MatchStatus::Pending => (theme::TEXT_MUTED, "⏳"),
                                };
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(st).size(13.0).color(sc));
                                });
                            });
                        });

                    if resp.response.interact(Sense::click()).clicked() {
                        clicked_match_id = Some(m.id.clone());
                    }
                    ui.add_space(4.0);
                }
            });
    }

    if let Some(id) = clicked_match_id {
        app.select_match(&id);
    }
}

fn render_elimination_bracket(app: &mut TourviaApp, ui: &mut Ui) {
    let zoom = app.bracket_zoom;
    let num_rounds = app.rounds.len();
    let first_round_matches = app.matches.iter().filter(|m| m.round_id == app.rounds[0].id).count();

    let card_w = MATCH_CARD_WIDTH * zoom;
    let card_h = MATCH_CARD_HEIGHT * zoom;
    let h_gap = ROUND_HORIZONTAL_GAP * zoom;
    let v_gap = MATCH_VERTICAL_GAP * zoom;
    let header_h = ROUND_HEADER_HEIGHT * zoom;
    let half_h = card_h / 2.0;

    let total_width = num_rounds as f32 * (card_w + h_gap) + h_gap;
    let total_height = first_round_matches as f32 * (card_h + v_gap) + header_h + 40.0 * zoom;

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                Vec2::new(total_width.max(600.0), total_height.max(400.0)),
                Sense::click(),
            );

            let origin = response.rect.left_top();
            let mut clicked_match_id = None;
            let first_round_total_height = first_round_matches as f32 * (card_h + v_gap);

            for (round_idx, round) in app.rounds.iter().enumerate() {
                let round_x = origin.x + h_gap / 2.0 + round_idx as f32 * (card_w + h_gap);

                // Round Header
                let header_rect = Rect::from_min_size(Pos2::new(round_x, origin.y), Vec2::new(card_w, header_h));
                painter.text(
                    header_rect.center(),
                    Align2::CENTER_CENTER,
                    &round.name,
                    FontId::new(14.0 * zoom, FontFamily::Proportional),
                    theme::TEXT_SECONDARY,
                );

                let round_matches: Vec<_> = app.matches.iter().filter(|m| m.round_id == round.id).collect();
                let matches_in_round = round_matches.len();
                let vertical_spacing = if matches_in_round > 0 { first_round_total_height / matches_in_round as f32 } else { 0.0 };

                for (match_idx, m) in round_matches.iter().enumerate() {
                    let match_y = origin.y + header_h + match_idx as f32 * vertical_spacing + (vertical_spacing - card_h) / 2.0;
                    let card_rect = Rect::from_min_size(Pos2::new(round_x, match_y), Vec2::new(card_w, card_h));

                    let is_selected = Some(&m.id) == app.selected_match.as_ref();
                    
                    // Card background & Border
                    painter.rect_filled(card_rect, (4.0 * zoom) as u8, if is_selected { theme::BG_ELEVATED } else { theme::BG_CARD });
                    painter.rect_stroke(card_rect, (4.0 * zoom) as u8, Stroke::new(if is_selected { 2.0 } else { 1.0 }, if is_selected { theme::ACCENT_BRONZE } else { theme::BORDER }), StrokeKind::Inside);

                    // Divider line
                    let div_y = card_rect.min.y + half_h;
                    painter.line_segment([Pos2::new(card_rect.min.x, div_y), Pos2::new(card_rect.max.x, div_y)], Stroke::new(1.0, theme::BORDER));

                    let score_box_w = 30.0 * zoom;

                    // --- Player 1 (Top Half) ---
                    let p1_name = if m.player1_name.is_empty() { "TBD" } else { &m.player1_name };
                    let p1_win = m.winner_id.is_some() && m.player1_id == m.winner_id;
                    let p1_color = if p1_win { theme::TEXT_PRIMARY } else if m.player1_name == "BYE" { theme::TEXT_MUTED } else { theme::TEXT_SECONDARY };
                    
                    painter.text(Pos2::new(card_rect.min.x + 8.0 * zoom, card_rect.min.y + half_h / 2.0), Align2::LEFT_CENTER, p1_name, FontId::new(12.0 * zoom, FontFamily::Proportional), p1_color);
                    
                    if m.status == MatchStatus::Completed || m.status == MatchStatus::InProgress {
                        let score1_rect = Rect::from_min_size(Pos2::new(card_rect.max.x - score_box_w, card_rect.min.y), Vec2::new(score_box_w, half_h));
                        painter.rect_filled(score1_rect, CornerRadius { nw: 0, ne: (4.0*zoom) as u8, sw: 0, se: 0 }, theme::BG_PANEL);
                        painter.line_segment([Pos2::new(score1_rect.min.x, score1_rect.min.y), Pos2::new(score1_rect.min.x, score1_rect.max.y)], Stroke::new(1.0, theme::BORDER));
                        painter.text(score1_rect.center(), Align2::CENTER_CENTER, &m.score1.to_string(), FontId::new(12.0 * zoom, FontFamily::Proportional), if p1_win { theme::SUCCESS } else { theme::TEXT_MUTED });
                        
                        if p1_win {
                            painter.line_segment([Pos2::new(score1_rect.max.x - 2.0*zoom, score1_rect.min.y + 2.0*zoom), Pos2::new(score1_rect.max.x - 2.0*zoom, score1_rect.max.y - 2.0*zoom)], Stroke::new(3.0 * zoom, theme::SUCCESS));
                        }
                    }

                    // --- Player 2 (Bottom Half) ---
                    let p2_name = if m.player2_name.is_empty() { "TBD" } else { &m.player2_name };
                    let p2_win = m.winner_id.is_some() && m.player2_id == m.winner_id;
                    let p2_color = if p2_win { theme::TEXT_PRIMARY } else if m.player2_name == "BYE" { theme::TEXT_MUTED } else { theme::TEXT_SECONDARY };
                    
                    painter.text(Pos2::new(card_rect.min.x + 8.0 * zoom, div_y + half_h / 2.0), Align2::LEFT_CENTER, p2_name, FontId::new(12.0 * zoom, FontFamily::Proportional), p2_color);
                    
                    if m.status == MatchStatus::Completed || m.status == MatchStatus::InProgress {
                        let score2_rect = Rect::from_min_size(Pos2::new(card_rect.max.x - score_box_w, div_y), Vec2::new(score_box_w, half_h));
                        painter.rect_filled(score2_rect, CornerRadius { nw: 0, ne: 0, sw: 0, se: (4.0*zoom) as u8 }, theme::BG_PANEL);
                        painter.line_segment([Pos2::new(score2_rect.min.x, score2_rect.min.y), Pos2::new(score2_rect.min.x, score2_rect.max.y)], Stroke::new(1.0, theme::BORDER));
                        painter.text(score2_rect.center(), Align2::CENTER_CENTER, &m.score2.to_string(), FontId::new(12.0 * zoom, FontFamily::Proportional), if p2_win { theme::SUCCESS } else { theme::TEXT_MUTED });

                        if p2_win {
                            painter.line_segment([Pos2::new(score2_rect.max.x - 2.0*zoom, score2_rect.min.y + 2.0*zoom), Pos2::new(score2_rect.max.x - 2.0*zoom, score2_rect.max.y - 2.0*zoom)], Stroke::new(3.0 * zoom, theme::SUCCESS));
                        }
                    }

                    if m.status == MatchStatus::Bye {
                        painter.text(Pos2::new(card_rect.max.x - 4.0*zoom, card_rect.min.y + half_h), Align2::RIGHT_CENTER, "BYE", FontId::new(10.0 * zoom, FontFamily::Proportional), theme::TEXT_MUTED);
                    }

                    // Click detection
                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            if card_rect.contains(pos) {
                                clicked_match_id = Some(m.id.clone());
                            }
                        }
                    }

                    // Right Angle Connector Lines (Challonge style)
                    if round_idx < num_rounds - 1 {
                        if let Some(ref next_match_id) = m.next_match_id {
                            let next_round = &app.rounds[round_idx + 1];
                            let next_round_matches: Vec<_> = app.matches.iter().filter(|nm| nm.round_id == next_round.id).collect();

                            if let Some((next_idx, _)) = next_round_matches.iter().enumerate().find(|(_, nm)| nm.id == *next_match_id) {
                                let next_vs = if next_round_matches.len() > 0 { first_round_total_height / next_round_matches.len() as f32 } else { 0.0 };
                                let next_x = origin.x + h_gap / 2.0 + (round_idx + 1) as f32 * (card_w + h_gap);
                                let next_y = origin.y + header_h + next_idx as f32 * next_vs + (next_vs - card_h) / 2.0;

                                let start = Pos2::new(card_rect.max.x, card_rect.center().y);
                                let end = Pos2::new(next_x, next_y + half_h);
                                let mid_x = start.x + h_gap / 2.0;
                                let stroke = Stroke::new(2.0 * zoom, theme::CONNECTOR_LINE);

                                // Draw orthogonal (right angle) line
                                painter.line_segment([start, Pos2::new(mid_x, start.y)], stroke);
                                painter.line_segment([Pos2::new(mid_x, start.y), Pos2::new(mid_x, end.y)], stroke);
                                painter.line_segment([Pos2::new(mid_x, end.y), end], stroke);
                            }
                        }
                    }
                }
            }

            if let Some(id) = clicked_match_id {
                app.select_match(&id);
            }
        });
}
