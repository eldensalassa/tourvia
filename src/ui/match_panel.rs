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
                        ui.label(theme::heading_text("MATCH DETAILS").size(22.0));
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
                
            let (rect, _) = ui.allocate_exact_size(Vec2::new(modal_width, 2.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 0.0, theme::ACCENT_BRONZE());

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
                                let rect = Rect::from_center_size(Pos2::new(cx, logo_cy), Vec2::new(40.0, 40.0));
                                painter.rect_stroke(rect, 4.0, Stroke::new(1.0, theme::BORDER_SUBTLE()), StrokeKind::Inside);
                                painter.text(Pos2::new(cx, logo_cy), Align2::CENTER_CENTER, "?", FontId::new(20.0, FontFamily::Name("Impact".into())), theme::TEXT_MUTED());
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
                            vp.text(Pos2::new(col1_cx, name_y), Align2::CENTER_TOP, p1, FontId::new(14.0, FontFamily::Proportional), p1_color);
                            vp.text(Pos2::new(col2_cx, name_y), Align2::CENTER_TOP, p2, FontId::new(14.0, FontFamily::Proportional), p2_color);

                            // Vertical separator lines
                            let sep_color = theme::BORDER_SUBTLE().linear_multiply(0.5);
                            let sep_x1 = vs_rect.left() + vs_card_width * 0.40;
                            let sep_x2 = vs_rect.left() + vs_card_width * 0.60;
                            vp.line_segment([Pos2::new(sep_x1, vs_rect.top() + 12.0), Pos2::new(sep_x1, vs_rect.bottom() - 12.0)], Stroke::new(0.5, sep_color));
                            vp.line_segment([Pos2::new(sep_x2, vs_rect.top() + 12.0), Pos2::new(sep_x2, vs_rect.bottom() - 12.0)], Stroke::new(0.5, sep_color));

                            // Center: VS or Score
                            if m.status == MatchStatus::Completed {
                                vp.text(Pos2::new(center_cx, logo_cy), Align2::CENTER_CENTER, format!("{}  :  {}", m.score1, m.score2), FontId::new(32.0, FontFamily::Name("Impact".into())), theme::TEXT_PRIMARY());
                            } else {
                                vp.text(Pos2::new(center_cx, logo_cy), Align2::CENTER_CENTER, "VS", FontId::new(28.0, FontFamily::Name("Impact".into())), theme::ACCENT_BRONZE());
                            }

                            // Scores below names (for completed matches)
                            if m.status == MatchStatus::Completed {
                                let rect_w = 36.0;
                                let rect_h = 28.0;
                                let s1_bg = if w1 { theme::SUCCESS().linear_multiply(0.15) } else { theme::BG_ELEVATED() };
                                let s2_bg = if w2 { theme::SUCCESS().linear_multiply(0.15) } else { theme::BG_ELEVATED() };
                                
                                let rect1 = Rect::from_center_size(Pos2::new(col1_cx, score_y), Vec2::new(rect_w, rect_h));
                                vp.rect_filled(rect1, 4.0, s1_bg);
                                vp.rect_stroke(rect1, 4.0, Stroke::new(1.0, if w1 { theme::SUCCESS() } else { theme::BORDER_SUBTLE() }), StrokeKind::Inside);
                                vp.text(Pos2::new(col1_cx, score_y), Align2::CENTER_CENTER, m.score1.to_string(), FontId::new(18.0, FontFamily::Name("Impact".into())), p1_color);

                                let rect2 = Rect::from_center_size(Pos2::new(col2_cx, score_y), Vec2::new(rect_w, rect_h));
                                vp.rect_filled(rect2, 4.0, s2_bg);
                                vp.rect_stroke(rect2, 4.0, Stroke::new(1.0, if w2 { theme::SUCCESS() } else { theme::BORDER_SUBTLE() }), StrokeKind::Inside);
                                vp.text(Pos2::new(col2_cx, score_y), Align2::CENTER_CENTER, m.score2.to_string(), FontId::new(18.0, FontFamily::Name("Impact".into())), p2_color);
                            }

                            ui.add_space(16.0);

                            // === Score Entry (for in-progress matches) ===
                            if m.status == MatchStatus::Pending {
                                ui.separator();
                                ui.add_space(12.0);
                                
                                let players_ready = m.player1_id.is_some() && m.player2_id.is_some();
                                let btn_text = if players_ready { "START MATCH" } else { "WAITING FOR OPPONENTS" };
                                let btn_color = if players_ready { theme::SUCCESS() } else { theme::TEXT_MUTED().linear_multiply(0.2) };
                                
                                let btn = egui::Button::new(RichText::new(btn_text).font(egui::FontId::new(24.0, egui::FontFamily::Name("Impact".into()))).color(theme::BG_DARK()))
                                    .fill(btn_color).corner_radius(theme::button_rounding())
                                    .min_size(Vec2::new(ui.available_width(), 48.0));
                                    
                                if ui.add_enabled(players_ready, btn).clicked() {
                                    app.start_match();
                                }
                            } else if m.status == MatchStatus::InProgress {
                                ui.separator();
                                ui.add_space(12.0);
                                
                                ui.horizontal(|ui| {
                                    ui.label(theme::heading_text("LIVE MATCH CONTROLS").size(16.0));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let status_text = if app.show_broadcast_window { "OVERLAY: ON" } else { "OVERLAY: OFF" };
                                        let bg_color = if app.show_broadcast_window { theme::SUCCESS() } else { theme::TEXT_MUTED() };
                                        let toggle_btn = egui::Button::new(RichText::new(status_text).color(theme::BG_DARK()).strong())
                                            .fill(bg_color)
                                            .corner_radius(theme::badge_rounding());
                                        if ui.add(toggle_btn).clicked() {
                                            app.show_broadcast_window = !app.show_broadcast_window;
                                        }
                                    });
                                });
                                ui.add_space(16.0);

                                // Timer Controls
                                egui::Frame::new().fill(theme::BG_ELEVATED()).corner_radius(4).inner_margin(egui::Margin::symmetric(16, 12)).show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("TIMER").size(12.0).color(theme::TEXT_SECONDARY()).strong());
                                        ui.add_space(8.0);
                                        let mins = app.broadcast_timer_seconds / 60;
                                        let secs = app.broadcast_timer_seconds % 60;
                                        ui.label(RichText::new(format!("{:02}:{:02}", mins, secs)).font(egui::FontId::new(28.0, egui::FontFamily::Name("Impact".into()))).color(theme::ACCENT_BRONZE()));
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let btn_reset = egui::Button::new(RichText::new("RESET").size(12.0).color(theme::TEXT_PRIMARY())).fill(theme::BG_CARD()).corner_radius(theme::button_rounding()).min_size(Vec2::new(60.0, 32.0));
                                            if ui.add(btn_reset).clicked() {
                                                app.broadcast_timer_seconds = 0;
                                                app.broadcast_timer_running = false;
                                            }
                                            let play_text = if app.broadcast_timer_running { "PAUSE" } else { "START" };
                                            let play_color = if app.broadcast_timer_running { theme::WARNING() } else { theme::SUCCESS() };
                                            let btn_play = egui::Button::new(RichText::new(play_text).size(12.0).color(theme::BG_DARK()).strong()).fill(play_color).corner_radius(theme::button_rounding()).min_size(Vec2::new(60.0, 32.0));
                                            if ui.add(btn_play).clicked() {
                                                app.broadcast_timer_running = !app.broadcast_timer_running;
                                                if app.broadcast_timer_running {
                                                    app.broadcast_timer_last_tick = None;
                                                }
                                            }
                                        });
                                    });
                                });
                                ui.add_space(16.0);

                                // Live Score Controls
                                ui.columns(2, |cols| {
                                    // P1
                                    cols[0].vertical_centered(|ui| {
                                        ui.label(RichText::new(p1).size(14.0).color(theme::TEXT_SECONDARY()).strong());
                                        ui.add_space(8.0);
                                        ui.horizontal(|ui| {
                                            ui.add_space(ui.available_width() / 2.0 - 56.0); // Center align manually
                                            let btn_minus = egui::Button::new(RichText::new("-").font(egui::FontId::new(24.0, egui::FontFamily::Name("Impact".into())))).fill(theme::BG_ELEVATED()).corner_radius(theme::button_rounding()).min_size(Vec2::new(36.0, 36.0));
                                            if ui.add(btn_minus).clicked() { app.update_live_score(-1, 0); }
                                            ui.add_space(8.0);
                                            ui.label(RichText::new(m.score1.to_string()).font(egui::FontId::new(36.0, egui::FontFamily::Name("Impact".into()))).color(theme::TEXT_PRIMARY()));
                                            ui.add_space(8.0);
                                            let btn_plus = egui::Button::new(RichText::new("+").font(egui::FontId::new(24.0, egui::FontFamily::Name("Impact".into())))).fill(theme::BG_ELEVATED()).corner_radius(theme::button_rounding()).min_size(Vec2::new(36.0, 36.0));
                                            if ui.add(btn_plus).clicked() { app.update_live_score(1, 0); }
                                        });
                                    });
                                    // P2
                                    cols[1].vertical_centered(|ui| {
                                        ui.label(RichText::new(p2).size(14.0).color(theme::TEXT_SECONDARY()).strong());
                                        ui.add_space(8.0);
                                        ui.horizontal(|ui| {
                                            ui.add_space(ui.available_width() / 2.0 - 56.0); // Center align manually
                                            let btn_minus = egui::Button::new(RichText::new("-").font(egui::FontId::new(24.0, egui::FontFamily::Name("Impact".into())))).fill(theme::BG_ELEVATED()).corner_radius(theme::button_rounding()).min_size(Vec2::new(36.0, 36.0));
                                            if ui.add(btn_minus).clicked() { app.update_live_score(0, -1); }
                                            ui.add_space(8.0);
                                            ui.label(RichText::new(m.score2.to_string()).font(egui::FontId::new(36.0, egui::FontFamily::Name("Impact".into()))).color(theme::TEXT_PRIMARY()));
                                            ui.add_space(8.0);
                                            let btn_plus = egui::Button::new(RichText::new("+").font(egui::FontId::new(24.0, egui::FontFamily::Name("Impact".into())))).fill(theme::BG_ELEVATED()).corner_radius(theme::button_rounding()).min_size(Vec2::new(36.0, 36.0));
                                            if ui.add(btn_plus).clicked() { app.update_live_score(0, 1); }
                                        });
                                    });
                                });

                                ui.add_space(16.0);

                                if ui.add(egui::Button::new(RichText::new("END MATCH & SAVE").font(egui::FontId::new(20.0, egui::FontFamily::Name("Impact".into()))).color(egui::Color32::WHITE))
                                    .fill(theme::ERROR().linear_multiply(0.85)).corner_radius(theme::button_rounding())
                                    .min_size(Vec2::new(ui.available_width(), 40.0))).clicked() {
                                    app.end_match();
                                }
                            }
                        }
                    }
                });
        });

    app.show_match_modal = is_open;
}
