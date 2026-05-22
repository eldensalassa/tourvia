use egui::{Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use crate::app::TourviaApp;
use crate::ui::theme;

pub fn render(app: &mut TourviaApp, ui: &mut Ui) {
    let tid = match &app.active_tournament {
        Some(t) => t.id.clone(),
        None => {
            ui.label(theme::label_text("No active tournament."));
            return;
        }
    };

    let stats = match app.services.match_service.get_tournament_stats(&tid) {
        Ok(s) => s,
        Err(e) => {
            ui.label(egui::RichText::new(e).color(theme::ERROR));
            return;
        }
    };

    ui.horizontal(|ui| {
        ui.label(theme::subheading_text("Standings & Progress"));
        
        let progress = if stats.total_matches > 0 { stats.completed_matches as f32 / stats.total_matches as f32 } else { 0.0 };
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(&format!("{}% Complete", (progress * 100.0) as i32)).size(13.0).color(theme::TEXT_MUTED));
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
        ui.label(egui::RichText::new(format!("{} Completed", stats.completed_matches)).color(theme::SUCCESS));
        ui.label(egui::RichText::new(format!("{} In Progress", stats.in_progress_matches)).color(theme::ACCENT_BRONZE_LIGHT));
        ui.label(egui::RichText::new(format!("{} Pending", stats.pending_matches)).color(theme::TEXT_MUTED));
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
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(40.0))          // Rank
                .column(Column::remainder().at_least(150.0)) // Participant
                .column(Column::exact(60.0))          // Pts
                .column(Column::exact(40.0))          // W
                .column(Column::exact(40.0))          // D
                .column(Column::exact(40.0))          // L
                .column(Column::exact(60.0))          // Win %
                .min_scrolled_height(0.0);

            table.header(30.0, |mut header| {
                header.col(|ui| { ui.label(theme::label_text("Rank")); });
                header.col(|ui| { ui.label(theme::label_text("Participant")); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("Pts")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("W")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("D")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("L")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("Win %")); }); });
            })
            .body(|mut body| {
                for (idx, (name, points, wins, losses, draws)) in stats.standings.iter().enumerate() {
                    let total_games = wins + losses + draws;
                    let win_rate = if total_games > 0 { (*wins as f32 / total_games as f32) * 100.0 } else { 0.0 };

                    body.row(36.0, |mut row| {
                        row.col(|ui| {
                            let rank_color = match idx {
                                0 => theme::GOLD,
                                1 => egui::Color32::from_rgb(192, 192, 192),
                                2 => egui::Color32::from_rgb(205, 127, 50),
                                _ => theme::TEXT_MUTED,
                            };
                            ui.label(egui::RichText::new(format!("{}", idx + 1)).size(15.0).color(rank_color).strong());
                        });

                        row.col(|ui| {
                            ui.label(egui::RichText::new(name).size(14.0).color(theme::TEXT_PRIMARY));
                        });

                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(&points.to_string()).size(14.0).color(theme::TEXT_PRIMARY).strong());
                            });
                        });

                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(&wins.to_string()).size(14.0).color(theme::SUCCESS));
                            });
                        });

                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(&draws.to_string()).size(14.0).color(theme::TEXT_MUTED));
                            });
                        });

                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(&losses.to_string()).size(14.0).color(theme::ERROR));
                            });
                        });

                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let wr_str = if total_games > 0 { format!("{:.0}%", win_rate) } else { "-".to_string() };
                                ui.label(egui::RichText::new(wr_str).size(13.0).color(theme::TEXT_SECONDARY));
                            });
                        });
                    });
                }
            });
        });
}
