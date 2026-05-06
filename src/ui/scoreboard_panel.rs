use egui::{self, Ui, RichText, Vec2, Color32, Stroke};

use crate::app::TourviaApp;
use crate::domain::match_model::MatchStatus;
use crate::ui::theme;

pub fn render(app: &mut TourviaApp, ui: &mut Ui) {
    let tid = match &app.active_tournament {
        Some(t) => t.id.clone(),
        None => { ui.label(theme::label_text("No active tournament.")); return; }
    };

    let tournament_name = app.active_tournament.as_ref()
        .map(|t| t.name.clone()).unwrap_or_default();

    let live_matches: Vec<_> = app.matches.iter().filter(|m| m.status == MatchStatus::InProgress).cloned().collect();
    let upcoming_matches: Vec<_> = app.matches.iter()
        .filter(|m| m.status == MatchStatus::Pending && m.player1_id.is_some() && m.player2_id.is_some())
        .cloned().collect();

    // Empty state
    if live_matches.is_empty() && upcoming_matches.is_empty() && app.champion_name.is_none() {
        ui.add_space(60.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("📋").size(48.0));
            ui.add_space(12.0);
            ui.label(RichText::new("No matches yet.").size(16.0).color(theme::TEXT_MUTED));
            ui.add_space(6.0);
            ui.label(RichText::new("Generate a bracket first from the Participants tab.").size(13.0).color(theme::TEXT_MUTED));
        });
        let _ = tid; return;
    }

    let avail_w = ui.available_width();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        egui::Frame::new()
            .fill(theme::BG_PANEL)
            .stroke(theme::card_stroke())
            .corner_radius(14.0)
            .inner_margin(egui::Margin::same(18))
            .show(ui, |ui| {
                // ─── Header: Tournament Name ─────────────────────────
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(tournament_name.to_uppercase())
                        .size(24.0).color(theme::TEXT_PRIMARY).strong());
                    ui.add_space(6.0);

                    if let Some(live) = live_matches.first() {
                        let round_name = app.rounds.iter().find(|r| r.id == live.round_id)
                            .map(|r| r.name.clone()).unwrap_or_default();
                        ui.label(RichText::new(format!("{} — BERLANGSUNG", round_name.to_uppercase()))
                            .size(12.0).color(theme::ACCENT_BRONZE_LIGHT));
                    } else if let Some(ref champ) = app.champion_name.clone() {
                        ui.label(RichText::new(format!("🏆 JUARA: {}", champ.to_uppercase()))
                            .size(13.0).color(theme::GOLD));
                    }
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(18.0);

                // ─── Live Match Card ─────────────────────────────────
                if let Some(live) = live_matches.first() {
                    let p1 = if live.player1_name.is_empty() { "TBD".to_string() } else { live.player1_name.clone() };
                    let p2 = if live.player2_name.is_empty() { "TBD".to_string() } else { live.player2_name.clone() };

                    ui.label(RichText::new("PERTANDINGAN LIVE").size(12.0).color(theme::ACCENT_BRONZE_LIGHT).strong());
                    ui.add_space(10.0);

                    egui::Frame::new()
                        .fill(Color32::from_rgb(18, 22, 38))
                        .stroke(Stroke::new(1.5, Color32::from_rgb(60, 80, 140)))
                        .corner_radius(16)
                        .inner_margin(egui::Margin::symmetric(24, 22))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let logo_size = 64.0;
                            let col_w = 220.0;
                            let score_w = 240.0;
                            let total_w = col_w * 2.0 + score_w;
                            let row_h = 160.0;

                            ui.vertical_centered(|ui| {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(total_w, row_h),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                // ── Team 1 ──
                                ui.allocate_ui_with_layout(
                                    Vec2::new(col_w, row_h),
                                    egui::Layout::top_down(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(10.0);
                                        if let Some(ref id) = live.player1_id {
                                            if let Some(tex) = app.logo_textures.get(id) {
                                                ui.add(egui::Image::new(tex)
                                                    .fit_to_exact_size(Vec2::new(logo_size, logo_size))
                                                    .corner_radius(10));
                                            } else {
                                                logo_placeholder(ui, logo_size);
                                            }
                                        } else {
                                            logo_placeholder(ui, logo_size);
                                        }
                                        ui.add_space(10.0);
                                        ui.label(RichText::new(&p1).size(18.0).color(theme::TEXT_PRIMARY).strong());
                                        if let Some(p) = app.participants.iter().find(|p| p.name == p1) {
                                            ui.label(RichText::new(format!("Seed #{}", p.seed)).size(11.0).color(theme::TEXT_MUTED));
                                        }
                                    }
                                );

                                // ── Score Center ──
                                ui.allocate_ui_with_layout(
                                    Vec2::new(score_w, row_h),
                                    egui::Layout::top_down(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(20.0);
                                        ui.label(RichText::new(format!("{}  :  {}", live.score1, live.score2))
                                            .size(48.0).color(theme::TEXT_PRIMARY).strong());
                                        ui.add_space(4.0);
                                        ui.label(RichText::new("SKOR PERTANDINGAN").size(11.0).color(theme::TEXT_MUTED));
                                        ui.add_space(10.0);
                                        if app.score_submitted {
                                            egui::Frame::new()
                                                .fill(Color32::from_rgb(40, 30, 10))
                                                .stroke(Stroke::new(1.0, theme::GOLD))
                                                .corner_radius(20)
                                                .inner_margin(egui::Margin::symmetric(14, 5))
                                                .show(ui, |ui| {
                                                    ui.label(RichText::new("✓  HASIL AKHIR").size(12.0).color(theme::GOLD).strong());
                                                });
                                        } else {
                                            egui::Frame::new()
                                                .fill(Color32::from_rgb(20, 80, 40))
                                                .stroke(Stroke::new(1.0, theme::SUCCESS))
                                                .corner_radius(20)
                                                .inner_margin(egui::Margin::symmetric(14, 5))
                                                .show(ui, |ui| {
                                                    ui.label(RichText::new("● LIVE").size(12.0).color(theme::SUCCESS).strong());
                                                });
                                        }
                                    }
                                );

                                // ── Team 2 ──
                                ui.allocate_ui_with_layout(
                                    Vec2::new(col_w, row_h),
                                    egui::Layout::top_down(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(10.0);
                                        if let Some(ref id) = live.player2_id {
                                            if let Some(tex) = app.logo_textures.get(id) {
                                                ui.add(egui::Image::new(tex)
                                                    .fit_to_exact_size(Vec2::new(logo_size, logo_size))
                                                    .corner_radius(10));
                                            } else {
                                                logo_placeholder(ui, logo_size);
                                            }
                                        } else {
                                            logo_placeholder(ui, logo_size);
                                        }
                                        ui.add_space(10.0);
                                        ui.label(RichText::new(&p2).size(18.0).color(theme::TEXT_PRIMARY).strong());
                                        if let Some(p) = app.participants.iter().find(|p| p.name == p2) {
                                            ui.label(RichText::new(format!("Seed #{}", p.seed)).size(11.0).color(theme::TEXT_MUTED));
                                        }
                                    }
                                );
                            });
                            });
                        });

                    ui.add_space(22.0);

                    // ─── Upcoming matches (2 columns like reference) ──
                    if !upcoming_matches.is_empty() {
                        let next_two: Vec<_> = upcoming_matches.iter().take(2).collect();

                        ui.horizontal(|ui| {
                            let col_w = (avail_w - 16.0) / 2.0;

                            for (i, m) in next_two.iter().enumerate() {
                                let p1 = if m.player1_name.is_empty() { "TBD" } else { &m.player1_name };
                                let p2 = if m.player2_name.is_empty() { "TBD" } else { &m.player2_name };
                                let round_name = app.rounds.iter().find(|r| r.id == m.round_id)
                                    .map(|r| r.name.clone()).unwrap_or_default();

                                ui.vertical(|ui| {
                                    ui.label(RichText::new("PERTANDINGAN BERIKUTNYA")
                                        .size(10.0).color(theme::ACCENT_BRONZE_LIGHT).strong());
                                    ui.add_space(6.0);

                                    egui::Frame::new()
                                        .fill(Color32::from_rgb(18, 22, 38))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(50, 65, 110)))
                                        .corner_radius(8)
                                        .inner_margin(egui::Margin::symmetric(16, 14))
                                        .show(ui, |ui| {
                                            ui.set_min_width(col_w - 8.0);
                                            ui.vertical_centered(|ui| {
                                                ui.label(RichText::new(format!("{} vs {}", p1, p2))
                                                    .size(14.0).color(theme::TEXT_PRIMARY).strong());
                                                ui.add_space(4.0);
                                                ui.label(RichText::new(format!("{} — {:02}:00", round_name, (m.match_order + 1) * 2))
                                                    .size(11.0).color(theme::ACCENT_BRONZE));
                                            });
                                        });
                                });

                                if i < next_two.len() - 1 {
                                    ui.add_space(8.0);
                                }
                            }
                        });

                        ui.add_space(20.0);
                    }

                    // ─── Score Input Bar ─────────────────────────────
                    egui::Frame::new()
                        .fill(Color32::from_rgb(22, 22, 26))
                        .stroke(Stroke::new(1.0, theme::BORDER))
                        .corner_radius(10)
                        .inner_margin(egui::Margin::symmetric(20, 18))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.vertical_centered(|ui| {
                                let input_w = 90.0;
                                let btn_w = 110.0;

                                if app.score_submitted {
                                    ui.add_space(4.0);
                                    ui.label(RichText::new("✅  Skor berhasil diupdate!")
                                        .size(13.0).color(theme::SUCCESS).strong());
                                    ui.add_space(10.0);
                                    ui.label(RichText::new("Penonton sudah melihat hasil? Lanjutkan ke pertandingan berikutnya.")
                                        .size(11.0).color(theme::TEXT_MUTED));
                                    ui.add_space(12.0);
                                    if ui.add(
                                        egui::Button::new(RichText::new("▶  Next Match").size(13.0).color(Color32::WHITE).strong())
                                            .fill(Color32::from_rgb(60, 80, 180))
                                            .min_size(Vec2::new(160.0, 36.0))
                                    ).clicked() {
                                        app.score_submitted = false;
                                        app.advance_to_next_match();
                                    }
                                    ui.add_space(4.0);
                                } else {
                                    // lebar total: SkorA(90) + gap(14) + :(16) + gap(14) + Update(110) + gap(14) + SkorB(90) = 348
                                    ui.allocate_ui_with_layout(
                                        Vec2::new(348.0, 60.0),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.vertical(|ui| {
                                                ui.label(RichText::new("Skor A").size(11.0).color(theme::TEXT_MUTED));
                                                let input = egui::TextEdit::singleline(&mut app.score_input[0])
                                                    .desired_width(input_w)
                                                    .font(egui::TextStyle::Heading);
                                                ui.add(input);
                                            });

                                            ui.add_space(14.0);
                                            ui.label(RichText::new(":").size(24.0).color(theme::TEXT_MUTED));
                                            ui.add_space(14.0);

                                            if ui.add(
                                                egui::Button::new(RichText::new("Update").size(13.0).color(Color32::WHITE).strong())
                                                    .fill(theme::SUCCESS)
                                                    .min_size(Vec2::new(btn_w, 34.0))
                                            ).clicked() {
                                                app.submit_match_score();
                                                app.score_submitted = true;
                                            }

                                            ui.add_space(14.0);
                                            ui.vertical(|ui| {
                                                ui.label(RichText::new("Skor B").size(11.0).color(theme::TEXT_MUTED));
                                                let input = egui::TextEdit::singleline(&mut app.score_input[1])
                                                    .desired_width(input_w)
                                                    .font(egui::TextStyle::Heading);
                                                ui.add(input);
                                            });
                                        }
                                    );
                                }
                            });
                        });

                    ui.add_space(8.0);
                    // Auto-select live match for score input
                    if app.selected_match.as_deref() != Some(live.id.as_str()) {
                        app.selected_match = Some(live.id.clone());
                    }
                } else if let Some(ref champion) = app.champion_name.clone() {
                    // ─── Champion full display ───────────────────────
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        let avail = ui.available_width();
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(avail, 120.0), egui::Sense::hover());
                        let painter = ui.painter();
                        painter.rect_filled(rect, 14.0, Color32::from_rgb(28, 20, 4));
                        painter.rect_stroke(rect, 14.0, Stroke::new(2.0, theme::GOLD), egui::epaint::StrokeKind::Inside);
                        painter.text(egui::pos2(rect.center().x, rect.min.y + 35.0), egui::Align2::CENTER_CENTER,
                            "🏆  TOURNAMENT CHAMPION  🏆", egui::FontId::new(14.0, egui::FontFamily::Proportional), theme::ACCENT_BRONZE_LIGHT);
                        painter.text(egui::pos2(rect.center().x, rect.min.y + 78.0), egui::Align2::CENTER_CENTER,
                            champion, egui::FontId::new(34.0, egui::FontFamily::Proportional), theme::GOLD);
                    });
                }
            });
    });

    let _ = tid;
}

fn logo_placeholder(ui: &mut Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(size, size), egui::Sense::hover());
    ui.painter().rect_filled(rect, 10.0, Color32::from_rgb(50, 55, 80));
    ui.painter().rect_stroke(rect, 10.0, Stroke::new(1.0, Color32::from_rgb(70, 75, 110)), egui::epaint::StrokeKind::Inside);
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER,
        "LOGO", egui::FontId::new(11.0, egui::FontFamily::Proportional), Color32::from_rgb(120, 125, 160));
}