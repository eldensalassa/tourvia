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
            ui.label(egui::RichText::new(e).color(theme::ERROR()));
            return;
        }
    };

    ui.horizontal(|ui| {
        ui.label(theme::subheading_text("Standings & Progress"));
        
        let progress = if stats.total_matches > 0 { stats.completed_matches as f32 / stats.total_matches as f32 } else { 0.0 };
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(&format!("{}% Complete", (progress * 100.0) as i32)).size(13.0).color(theme::TEXT_MUTED()));
        });
    });

    ui.add_space(16.0);

    // ─── Match Progress Bar ─────────────────────────────
    let bar_height = 12.0;
    let available_width = ui.available_width();
    let (rect, _response) = ui.allocate_exact_size(Vec2::new(available_width, bar_height), egui::Sense::hover());
    
    let progress = if stats.total_matches > 0 { stats.completed_matches as f32 / stats.total_matches as f32 } else { 0.0 };
    ui.painter().rect_filled(rect, bar_height / 2.0, theme::BG_PANEL());
    if progress > 0.0 {
        let mut fg_rect = rect;
        fg_rect.max.x = rect.min.x + (rect.width() * progress);
        ui.painter().rect_filled(fg_rect, bar_height / 2.0, theme::ACCENT_BRONZE());
    }
    
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{} Completed", stats.completed_matches)).color(theme::SUCCESS()));
        ui.label(egui::RichText::new(format!("{} In Progress", stats.in_progress_matches)).color(theme::ACCENT_BRONZE_LIGHT()));
        ui.label(egui::RichText::new(format!("{} Pending", stats.pending_matches)).color(theme::TEXT_MUTED()));
    });

    ui.add_space(24.0);

    // ─── Standings Table ───────────────────────────────
    if stats.standings.is_empty() {
        ui.label(theme::small_text("No participants."));
        return;
    }

    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(40.0))                 // Rank
                .column(Column::remainder().at_least(100.0)) // Team
                .column(Column::exact(45.0))                 // MP
                .column(Column::exact(60.0))                 // Pts/GW
                .column(Column::exact(45.0))                 // GD
                .column(Column::exact(45.0))                 // MW
                .column(Column::exact(45.0))                 // ML
                .column(Column::exact(55.0))                 // M WR
                .column(Column::exact(45.0))                 // GL
                .column(Column::exact(55.0))                 // G WR
                .column(Column::exact(16.0))                 // Spacer for right edge padding
                .min_scrolled_height(0.0);

            table.header(30.0, |mut header| {
                header.col(|ui| { ui.label(theme::label_text("Rank")); });
                header.col(|ui| { ui.label(theme::label_text("Team")); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("MP")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("Pts/GW")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("GD")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("MW")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("ML")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("M WR")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("GL")); }); });
                header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("G WR")); }); });
                header.col(|_ui| { /* spacer */ });
            })
            .body(|mut body| {
                for (idx, standing) in stats.standings.iter().enumerate() {
                    let match_win_rate = if standing.matches_played > 0 { (standing.matches_won as f32 / standing.matches_played as f32) * 100.0 } else { 0.0 };
                    let total_games = standing.games_won + standing.games_lost;
                    let game_win_rate = if total_games > 0 { (standing.games_won as f32 / total_games as f32) * 100.0 } else { 0.0 };
                    let game_diff = standing.games_won - standing.games_lost;

                    body.row(36.0, |mut row| {
                        row.col(|ui| {
                            let rank_color = match idx {
                                0 => theme::GOLD(),
                                1 => egui::Color32::from_rgb(192, 192, 192),
                                2 => egui::Color32::from_rgb(205, 127, 50),
                                _ => theme::TEXT_MUTED(),
                            };
                            ui.label(egui::RichText::new(format!("{}", idx + 1)).size(15.0).color(rank_color).strong());
                        });

                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if let Some(texture) = app.logo_textures.get(&standing.id) {
                                    ui.add(egui::Image::new(texture).fit_to_exact_size(Vec2::new(18.0, 18.0)).corner_radius(2.0));
                                    ui.add_space(4.0);
                                }
                                ui.label(egui::RichText::new(&standing.name).size(14.0).color(theme::TEXT_PRIMARY()));
                            });
                        });

                        // MP
                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(&standing.matches_played.to_string()).size(14.0).color(theme::TEXT_PRIMARY()));
                            });
                        });

                        // Pts/GW
                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(&standing.games_won.to_string()).size(14.0).color(theme::SUCCESS()));
                            });
                        });

                        // GD
                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let gd_str = if game_diff > 0 { format!("+{}", game_diff) } else { game_diff.to_string() };
                                let color = if game_diff > 0 { theme::SUCCESS() } else if game_diff < 0 { theme::ERROR() } else { theme::TEXT_MUTED() };
                                ui.label(egui::RichText::new(gd_str).size(14.0).color(color));
                            });
                        });

                        // MW
                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(&standing.matches_won.to_string()).size(14.0).color(theme::SUCCESS()));
                            });
                        });

                        // ML
                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(&standing.matches_lost.to_string()).size(14.0).color(theme::ERROR()));
                            });
                        });

                        // M WR
                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let wr_str = if standing.matches_played > 0 { format!("{:.0}%", match_win_rate) } else { "-".to_string() };
                                ui.label(egui::RichText::new(wr_str).size(13.0).color(theme::TEXT_SECONDARY()));
                            });
                        });

                        // GL
                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(&standing.games_lost.to_string()).size(14.0).color(theme::ERROR()));
                            });
                        });

                        // G WR
                        row.col(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let wr_str = if total_games > 0 { format!("{:.0}%", game_win_rate) } else { "-".to_string() };
                                ui.label(egui::RichText::new(wr_str).size(13.0).color(theme::TEXT_SECONDARY()));
                            });
                        });

                        // Spacer
                        row.col(|_ui| { /* space */ });
                    });
                }
            });
        });
}
