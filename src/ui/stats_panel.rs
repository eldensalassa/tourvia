use egui::{self, Ui, RichText, Vec2, Color32, Stroke};

use crate::app::TourviaApp;
use crate::services::match_service;
use crate::ui::theme;

pub fn render(app: &mut TourviaApp, ui: &mut Ui) {
    let tid = match &app.active_tournament {
        Some(t) => t.id.clone(),
        None => {
            ui.label(theme::label_text("No active tournament."));
            return;
        }
    };

    let stats = match match_service::get_tournament_stats(&app.db, &tid) {
        Ok(s) => s,
        Err(e) => {
            ui.label(RichText::new(e).color(theme::ERROR));
            return;
        }
    };

    ui.horizontal(|ui| {
        ui.label(theme::subheading_text("Standings & Progress"));
        
        let progress = if stats.total_matches > 0 { stats.completed_matches as f32 / stats.total_matches as f32 } else { 0.0 };
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(&format!("{}% Complete", (progress * 100.0) as i32)).size(13.0).color(theme::TEXT_MUTED));
        });
    });

    ui.add_space(16.0);

    // ─── Match Progress Bar ─────────────────────────────
    let bar_height = 12.0;
    let available_width = ui.available_width();
    let (rect, _response) = ui.allocate_exact_size(Vec2::new(available_width, bar_height), egui::Sense::hover());
    
    let progress = if stats.total_matches > 0 { stats.completed_matches as f32 / stats.total_matches as f32 } else { 0.0 };
    ui.painter().rect_filled(rect, bar_height / 2.0, theme::BG_PANEL);
    if progress > 0.0 {
        let mut fg_rect = rect;
        fg_rect.max.x = rect.min.x + (rect.width() * progress);
        ui.painter().rect_filled(fg_rect, bar_height / 2.0, theme::ACCENT_BRONZE);
    }
    
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{} Completed", stats.completed_matches)).color(theme::SUCCESS));
        ui.label(RichText::new(format!("{} In Progress", stats.in_progress_matches)).color(theme::ACCENT_BRONZE_LIGHT));
        ui.label(RichText::new(format!("{} Pending", stats.pending_matches)).color(theme::TEXT_MUTED));
    });

    ui.add_space(24.0);

    // ─── Standings Table ───────────────────────────────
    if stats.standings.is_empty() {
        ui.label(theme::small_text("No participants."));
        return;
    }

    egui::Frame::new()
        .fill(theme::BG_PANEL)
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .show(ui, |ui| {
            // Header
            egui::Frame::new()
                .fill(theme::BG_ELEVATED)
                .inner_margin(egui::Margin::symmetric(24, 12))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(Vec2::new(60.0, 16.0), egui::Layout::left_to_right(egui::Align::Center), |ui| { ui.label(theme::label_text("Rank")); });
                        ui.allocate_ui_with_layout(Vec2::new(400.0, 16.0), egui::Layout::left_to_right(egui::Align::Center), |ui| { ui.label(theme::label_text("Participant")); });
                        ui.allocate_ui_with_layout(Vec2::new(60.0, 16.0), egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("W")); });
                        ui.allocate_ui_with_layout(Vec2::new(60.0, 16.0), egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("L")); });
                        ui.allocate_ui_with_layout(Vec2::new(80.0, 16.0), egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("Win %")); });
                    });
                });

            // Rows
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, (name, wins, losses)) in stats.standings.iter().enumerate() {
                    let total_games = wins + losses;
                    let win_rate = if total_games > 0 { (*wins as f32 / total_games as f32) * 100.0 } else { 0.0 };
                    let bg = if idx % 2 == 0 { theme::BG_PANEL } else { theme::BG_CARD };

                    egui::Frame::new()
                        .fill(bg)
                        .inner_margin(egui::Margin::symmetric(24, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Rank
                                ui.allocate_ui_with_layout(Vec2::new(60.0, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    let rank_color = match idx {
                                        0 => theme::GOLD,
                                        1 => Color32::from_rgb(192, 192, 192),
                                        2 => Color32::from_rgb(205, 127, 50),
                                        _ => theme::TEXT_MUTED,
                                    };
                                    ui.label(RichText::new(format!("{}", idx + 1)).size(16.0).color(rank_color).strong());
                                });

                                // Name
                                ui.allocate_ui_with_layout(Vec2::new(400.0, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(name).size(15.0).color(theme::TEXT_PRIMARY));
                                });

                                // Wins
                                ui.allocate_ui_with_layout(Vec2::new(60.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(&wins.to_string()).size(14.0).color(theme::SUCCESS).strong());
                                });

                                // Losses
                                ui.allocate_ui_with_layout(Vec2::new(60.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(&losses.to_string()).size(14.0).color(theme::ERROR));
                                });

                                // Win Rate
                                ui.allocate_ui_with_layout(Vec2::new(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let wr_str = if total_games > 0 { format!("{:.0}%", win_rate) } else { "-".to_string() };
                                    ui.label(RichText::new(wr_str).size(14.0).color(theme::TEXT_SECONDARY));
                                });
                            });
                        });
                }
            });
        });
}
