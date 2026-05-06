use egui::{self, Ui, RichText, Vec2, Color32, Stroke};

use crate::app::TourviaApp;
use crate::services::match_service;
use crate::ui::theme;

pub fn render(app: &mut TourviaApp, ui: &mut Ui) {
    let tid = match &app.active_tournament {
        Some(t) => t.id.clone(),
        None => { ui.label(theme::label_text("No active tournament.")); return; }
    };

    let stats = match match_service::get_tournament_stats(&app.db, &tid) {
        Ok(s) => s,
        Err(e) => { ui.label(RichText::new(e).color(theme::ERROR)); return; }
    };

    // ─── Header + Progress ──────────────────────────────
    let progress = if stats.total_matches > 0 { stats.completed_matches as f32 / stats.total_matches as f32 } else { 0.0 };
    let pct = (progress * 100.0) as i32;

    ui.horizontal(|ui| {
        ui.label(theme::subheading_text("Standings"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(format!("{}% Complete", pct)).size(12.0).color(theme::TEXT_MUTED));
        });
    });
    ui.add_space(12.0);

    // ─── Progress Bar (segmented) ───────────────────────
    let bar_h = 8.0;
    let avail_w = ui.available_width();
    let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, bar_h), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(bar_rect, bar_h / 2.0, Color32::from_rgb(30, 30, 30));
    if progress > 0.0 {
        let mut fg = bar_rect;
        fg.max.x = bar_rect.min.x + bar_rect.width() * progress;
        // Gradient effect: two layers
        painter.rect_filled(fg, bar_h / 2.0, theme::ACCENT_BRONZE_DARK);
        let mut fg_inner = fg;
        fg_inner.max.y = bar_rect.min.y + bar_h * 0.55;
        painter.rect_filled(fg_inner, egui::CornerRadius { nw: 4, ne: 0, sw: 0, se: 0 }, theme::ACCENT_BRONZE);
    }

    ui.add_space(8.0);
    // Stat pills
    ui.horizontal(|ui| {
        stat_pill(ui, &format!("{} Done", stats.completed_matches), theme::SUCCESS, Color32::from_rgb(12, 28, 16));
        ui.add_space(6.0);
        stat_pill(ui, &format!("{} Live", stats.in_progress_matches), theme::ACCENT_BRONZE, Color32::from_rgb(28, 20, 8));
        ui.add_space(6.0);
        stat_pill(ui, &format!("{} Pending", stats.pending_matches), theme::TEXT_MUTED, Color32::from_rgb(24, 24, 24));
    });

    ui.add_space(20.0);

    if stats.standings.is_empty() {
        ui.label(theme::small_text("No participants."));
        return;
    }

    // ─── Standings Table ───────────────────────────────
    egui::Frame::new()
        .fill(theme::BG_PANEL)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(10)
        .show(ui, |ui| {
            // Table header
            egui::Frame::new()
                .fill(Color32::from_rgb(22, 22, 22))
                .inner_margin(egui::Margin::symmetric(20, 10))
                .corner_radius(egui::CornerRadius { nw: 10, ne: 10, sw: 0, se: 0 })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(Vec2::new(48.0, 16.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label(RichText::new("RANK").size(10.0).color(theme::ACCENT_BRONZE_LIGHT).strong());
                        });
                        ui.allocate_ui_with_layout(Vec2::new(ui.available_width() - 200.0, 16.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label(RichText::new("PARTICIPANT").size(10.0).color(theme::ACCENT_BRONZE_LIGHT).strong());
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.allocate_ui_with_layout(Vec2::new(65.0, 16.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new("WIN%").size(10.0).color(theme::ACCENT_BRONZE_LIGHT).strong());
                            });
                            ui.allocate_ui_with_layout(Vec2::new(50.0, 16.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new("L").size(10.0).color(theme::ACCENT_BRONZE_LIGHT).strong());
                            });
                            ui.allocate_ui_with_layout(Vec2::new(50.0, 16.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new("W").size(10.0).color(theme::ACCENT_BRONZE_LIGHT).strong());
                            });
                        });
                    });
                });

            // Table rows
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, (name, wins, losses)) in stats.standings.iter().enumerate() {
                    let total = wins + losses;
                    let win_rate = if total > 0 { (*wins as f32 / total as f32) * 100.0 } else { 0.0 };

                    let is_top3 = idx < 3;
                    let row_fill = if idx % 2 == 0 { theme::BG_PANEL } else { Color32::from_rgb(21, 21, 21) };

                    egui::Frame::new()
                        .fill(row_fill)
                        .inner_margin(egui::Margin::symmetric(20, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Rank badge
                                ui.allocate_ui_with_layout(Vec2::new(48.0, 24.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    let (badge_rect, _) = ui.allocate_exact_size(Vec2::new(26.0, 24.0), egui::Sense::hover());
                                    let (badge_fill, rank_color) = match idx {
                                        0 => (Color32::from_rgb(50, 40, 5), theme::GOLD),
                                        1 => (Color32::from_rgb(35, 35, 38), Color32::from_rgb(192, 192, 210)),
                                        2 => (Color32::from_rgb(38, 28, 18), Color32::from_rgb(205, 127, 50)),
                                        _ => (Color32::TRANSPARENT, theme::TEXT_MUTED),
                                    };
                                    ui.painter().rect_filled(badge_rect, 5.0, badge_fill);
                                    if is_top3 {
                                        ui.painter().rect_stroke(badge_rect, 5.0, Stroke::new(1.0, rank_color), egui::epaint::StrokeKind::Inside);
                                    }
                                    ui.painter().text(badge_rect.center(), egui::Align2::CENTER_CENTER,
                                        format!("{}", idx + 1), egui::FontId::new(13.0, egui::FontFamily::Proportional), rank_color);
                                });

                                // Logo + Name
                                let name_w = ui.available_width() - 200.0;
                                ui.allocate_ui_with_layout(Vec2::new(name_w.max(100.0), 24.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    // Try to find participant logo
                                    if let Some(p) = app.participants.iter().find(|p| p.name == *name) {
                                        if let Some(tex) = app.logo_textures.get(&p.id) {
                                            ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(22.0, 22.0)).corner_radius(4));
                                            ui.add_space(6.0);
                                        }
                                    }
                                    let name_color = if idx == 0 { theme::GOLD } else { theme::TEXT_PRIMARY };
                                    ui.label(RichText::new(name).size(14.0).color(name_color));
                                });

                                // Stats (right-aligned)
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Win rate bar + %
                                    ui.allocate_ui_with_layout(Vec2::new(65.0, 24.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let wr_str = if total > 0 { format!("{:.0}%", win_rate) } else { "-".to_string() };
                                        let wr_color = if win_rate >= 70.0 { theme::SUCCESS } else if win_rate >= 40.0 { theme::ACCENT_BRONZE } else { theme::TEXT_MUTED };
                                        ui.label(RichText::new(wr_str).size(13.0).color(wr_color).strong());
                                    });
                                    ui.allocate_ui_with_layout(Vec2::new(50.0, 24.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(losses.to_string()).size(14.0).color(theme::ERROR));
                                    });
                                    ui.allocate_ui_with_layout(Vec2::new(50.0, 24.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(wins.to_string()).size(14.0).color(theme::SUCCESS).strong());
                                    });
                                });
                            });
                        });

                    // Thin separator
                    if idx < stats.standings.len() - 1 {
                        let sep_w = ui.available_width();
                        let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(sep_w, 1.0), egui::Sense::hover());
                        ui.painter().rect_filled(sep_rect, 0.0, theme::BORDER_SUBTLE);
                    }
                }
            });
        });
}

fn stat_pill(ui: &mut Ui, text: &str, text_color: Color32, bg: Color32) {
    egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0, text_color.gamma_multiply(0.4)))
        .corner_radius(12)
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).color(text_color).strong());
        });
}
