use egui::{self, RichText, Vec2, Pos2, Rect, Stroke, StrokeKind, FontId, FontFamily, Align2};

use crate::app::TourviaApp;
use crate::domain::match_model::MatchStatus;
use crate::ui::theme;

pub fn render_modal(app: &mut TourviaApp, ctx: &egui::Context) {
    let mut is_open = app.show_match_modal;

    egui::Window::new("MatchDetailsModal")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(egui::Frame::new().fill(theme::BG_PANEL()).stroke(theme::card_stroke()).corner_radius(12).inner_margin(egui::Margin::same(0)))
        .show(ctx, |ui| {
            let modal_width = 420.0;
            ui.set_min_width(modal_width);

            // === Header bar (using egui widgets, not painter text) ===
            egui::Frame::new()
                .fill(theme::BG_ELEVATED())
                .corner_radius(egui::CornerRadius { nw: 12, ne: 12, sw: 0, se: 0 })
                .inner_margin(egui::Margin::symmetric(20, 12))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Match Details").size(17.0).color(theme::TEXT_PRIMARY()).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(RichText::new("X").size(14.0).color(theme::TEXT_MUTED()).strong())
                                .fill(egui::Color32::TRANSPARENT)
                                .frame(false)
                                .min_size(Vec2::new(24.0, 24.0))).clicked() {
                                is_open = false;
                            }
                        });
                    });
                });

            // === Body ===
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(24, 20))
                .show(ui, |ui| {
                    let selected_match = if let Some(ref match_id) = app.selected_match {
                        app.matches.iter().find(|m| m.id == *match_id).cloned()
                    } else { None };

                    match selected_match {
                        None => {
                            ui.label(RichText::new("No match selected.").size(14.0).color(theme::TEXT_MUTED()));
                        }
                        Some(m) => {
                            // Round info + status badge
                            let round_name = app.rounds.iter()
                                .find(|r| r.id == m.round_id)
                                .map(|r| r.name.as_str()).unwrap_or("Unknown");

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("{} — Match #{}", round_name, m.match_order + 1)).size(13.0).color(theme::TEXT_SECONDARY()));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let (badge_bg, badge_fg, badge_text) = match m.status {
                                        MatchStatus::Completed => (theme::SUCCESS().linear_multiply(0.15), theme::SUCCESS(), "COMPLETED"),
                                        MatchStatus::InProgress => (theme::ACCENT_BRONZE().linear_multiply(0.15), theme::ACCENT_BRONZE(), "IN PROGRESS"),
                                        MatchStatus::Bye => (theme::WARNING().linear_multiply(0.15), theme::WARNING(), "BYE"),
                                        MatchStatus::Pending => (theme::BG_ELEVATED(), theme::TEXT_MUTED(), "PENDING"),
                                    };
                                    egui::Frame::new().fill(badge_bg).corner_radius(4).inner_margin(egui::Margin::symmetric(8, 3)).show(ui, |ui| {
                                        ui.label(RichText::new(badge_text).size(10.0).color(badge_fg).strong());
                                    });
                                });
                            });

                            ui.add_space(16.0);

                            // === VS card using painter for pixel-perfect layout ===
                            let vs_card_width = ui.available_width();
                            let vs_card_height = if m.status == MatchStatus::Completed { 150.0 } else { 120.0 };
                            let (vs_rect, _) = ui.allocate_exact_size(Vec2::new(vs_card_width, vs_card_height), egui::Sense::hover());

                            let vp = ui.painter_at(vs_rect);

                            // VS card background
                            vp.rect_filled(vs_rect, 8.0, theme::BG_CARD());
                            vp.rect_stroke(vs_rect, 8.0, Stroke::new(0.5, theme::BORDER_SUBTLE()), StrokeKind::Inside);

                            let p1 = if m.player1_name.is_empty() { "TBD" } else { &m.player1_name };
                            let p2 = if m.player2_name.is_empty() { "TBD" } else { &m.player2_name };
                            let w1 = m.winner_id.is_some() && m.player1_id == m.winner_id;
                            let w2 = m.winner_id.is_some() && m.player2_id == m.winner_id;
                            let p1_color = if w1 { theme::SUCCESS() } else { theme::TEXT_PRIMARY() };
                            let p2_color = if w2 { theme::SUCCESS() } else { theme::TEXT_PRIMARY() };

                            // Column positions: exactly 25%, 50%, 75% of the card
                            let col1_cx = vs_rect.left() + vs_card_width * 0.25;
                            let center_cx = vs_rect.left() + vs_card_width * 0.50;
                            let col2_cx = vs_rect.left() + vs_card_width * 0.75;

                            let logo_box = 48.0;
                            let logo_cy = vs_rect.top() + 36.0;
                            let name_y = vs_rect.top() + 70.0;
                            let score_y = vs_rect.top() + 120.0;

                            // Helper: draw logo with aspect ratio preserved
                            let draw_logo = |painter: &egui::Painter, cx: f32, tex: &egui::TextureHandle| {
                                let tex_size = tex.size();
                                let tw = tex_size[0] as f32;
                                let th = tex_size[1] as f32;
                                if tw > 0.0 && th > 0.0 {
                                    let aspect = tw / th;
                                    let (dw, dh) = if aspect > 1.0 {
                                        (logo_box, logo_box / aspect)
                                    } else {
                                        (logo_box * aspect, logo_box)
                                    };
                                    let r = Rect::from_center_size(Pos2::new(cx, logo_cy), Vec2::new(dw, dh));
                                    painter.image(tex.id(), r, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), egui::Color32::WHITE);
                                }
                            };

                            // Helper: placeholder
                            let draw_placeholder = |painter: &egui::Painter, cx: f32| {
                                painter.circle_stroke(Pos2::new(cx, logo_cy), 20.0, Stroke::new(1.0, theme::BORDER_SUBTLE()));
                                painter.text(Pos2::new(cx, logo_cy), Align2::CENTER_CENTER, "?", FontId::new(16.0, FontFamily::Proportional), theme::TEXT_MUTED());
                            };

                            // Player 1 logo
                            let mut p1_logo = false;
                            if let Some(ref id) = m.player1_id {
                                if let Some(tex) = app.logo_textures.get(id) { draw_logo(&vp, col1_cx, tex); p1_logo = true; }
                            }
                            if !p1_logo { draw_placeholder(&vp, col1_cx); }

                            // Player 2 logo
                            let mut p2_logo = false;
                            if let Some(ref id) = m.player2_id {
                                if let Some(tex) = app.logo_textures.get(id) { draw_logo(&vp, col2_cx, tex); p2_logo = true; }
                            }
                            if !p2_logo { draw_placeholder(&vp, col2_cx); }

                            // Player names
                            vp.text(Pos2::new(col1_cx, name_y), Align2::CENTER_TOP, p1, FontId::new(13.0, FontFamily::Proportional), p1_color);
                            vp.text(Pos2::new(col2_cx, name_y), Align2::CENTER_TOP, p2, FontId::new(13.0, FontFamily::Proportional), p2_color);

                            // Vertical separator lines
                            let sep_color = theme::BORDER_SUBTLE().linear_multiply(0.5);
                            let sep_x1 = vs_rect.left() + vs_card_width * 0.40;
                            let sep_x2 = vs_rect.left() + vs_card_width * 0.60;
                            vp.line_segment([Pos2::new(sep_x1, vs_rect.top() + 12.0), Pos2::new(sep_x1, vs_rect.bottom() - 12.0)], Stroke::new(0.5, sep_color));
                            vp.line_segment([Pos2::new(sep_x2, vs_rect.top() + 12.0), Pos2::new(sep_x2, vs_rect.bottom() - 12.0)], Stroke::new(0.5, sep_color));

                            // Center: VS or Score
                            if m.status == MatchStatus::Completed {
                                vp.text(Pos2::new(center_cx, logo_cy), Align2::CENTER_CENTER, format!("{}  :  {}", m.score1, m.score2), FontId::new(26.0, FontFamily::Proportional), theme::TEXT_PRIMARY());
                            } else {
                                vp.text(Pos2::new(center_cx, logo_cy), Align2::CENTER_CENTER, "VS", FontId::new(18.0, FontFamily::Proportional), theme::TEXT_MUTED());
                            }

                            // Scores below names (for completed matches)
                            if m.status == MatchStatus::Completed {
                                let circle_r = 16.0;
                                let s1_bg = if w1 { theme::SUCCESS().linear_multiply(0.15) } else { theme::BG_ELEVATED() };
                                let s2_bg = if w2 { theme::SUCCESS().linear_multiply(0.15) } else { theme::BG_ELEVATED() };
                                vp.circle_filled(Pos2::new(col1_cx, score_y), circle_r, s1_bg);
                                vp.circle_stroke(Pos2::new(col1_cx, score_y), circle_r, Stroke::new(1.0, if w1 { theme::SUCCESS() } else { theme::BORDER_SUBTLE() }));
                                vp.text(Pos2::new(col1_cx, score_y), Align2::CENTER_CENTER, m.score1.to_string(), FontId::new(14.0, FontFamily::Proportional), p1_color);

                                vp.circle_filled(Pos2::new(col2_cx, score_y), circle_r, s2_bg);
                                vp.circle_stroke(Pos2::new(col2_cx, score_y), circle_r, Stroke::new(1.0, if w2 { theme::SUCCESS() } else { theme::BORDER_SUBTLE() }));
                                vp.text(Pos2::new(col2_cx, score_y), Align2::CENTER_CENTER, m.score2.to_string(), FontId::new(14.0, FontFamily::Proportional), p2_color);
                            }

                            ui.add_space(16.0);

                            // === Score Entry (for in-progress matches) ===
                            if m.status == MatchStatus::InProgress {
                                ui.separator();
                                ui.add_space(12.0);
                                ui.label(RichText::new("Report Score").size(14.0).color(theme::TEXT_PRIMARY()).strong());
                                ui.add_space(8.0);

                                ui.columns(2, |cols| {
                                    cols[0].vertical_centered(|ui| {
                                        ui.label(RichText::new(p1).size(12.0).color(theme::TEXT_SECONDARY()));
                                        ui.add_space(4.0);
                                        ui.add(egui::TextEdit::singleline(&mut app.score_input[0]).desired_width(100.0).hint_text("Score").horizontal_align(egui::Align::Center));
                                    });
                                    cols[1].vertical_centered(|ui| {
                                        ui.label(RichText::new(p2).size(12.0).color(theme::TEXT_SECONDARY()));
                                        ui.add_space(4.0);
                                        ui.add(egui::TextEdit::singleline(&mut app.score_input[1]).desired_width(100.0).hint_text("Score").horizontal_align(egui::Align::Center));
                                    });
                                });

                                ui.add_space(12.0);

                                if ui.add(egui::Button::new(RichText::new("Submit Match Result").size(13.0).color(theme::BG_DARK()).strong())
                                    .fill(theme::ACCENT_BRONZE()).corner_radius(theme::button_rounding())
                                    .min_size(Vec2::new(ui.available_width(), 36.0))).clicked() {
                                    app.submit_match_score();
                                }
                            }
                        }
                    }
                });
        });

    app.show_match_modal = is_open;
}
