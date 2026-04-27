use egui::{self, RichText, Vec2, Color32, Stroke};

use crate::app::TourviaApp;
use crate::domain::match_model::MatchStatus;
use crate::ui::theme;

pub fn render_modal(app: &mut TourviaApp, ctx: &egui::Context) {
    let mut is_open = app.show_match_modal;

    egui::Window::new("🎯 Match Details")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(egui::Frame::new().fill(theme::BG_PANEL).stroke(theme::card_stroke()).corner_radius(8).inner_margin(24))
        .show(ctx, |ui| {
            ui.set_min_width(320.0);

            let selected_match = if let Some(ref match_id) = app.selected_match {
                app.matches.iter().find(|m| m.id == *match_id).cloned()
            } else { None };

            match selected_match {
                None => {
                    ui.label("No match selected.");
                }
                Some(m) => {
                    let round_name = app.rounds.iter()
                        .find(|r| r.id == m.round_id)
                        .map(|r| r.name.as_str()).unwrap_or("Unknown");

                    ui.label(RichText::new(format!("{} — Match #{}", round_name, m.match_order + 1)).size(14.0).color(theme::TEXT_SECONDARY).strong());
                    ui.add_space(16.0);

                    let p1 = if m.player1_name.is_empty() { "TBD".to_string() } else { m.player1_name.clone() };
                    let p2 = if m.player2_name.is_empty() { "TBD".to_string() } else { m.player2_name.clone() };
                    
                    let w1 = m.winner_id.is_some() && m.player1_id == m.winner_id;
                    let w2 = m.winner_id.is_some() && m.player2_id == m.winner_id;

                    // VS Layout
                    ui.horizontal(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.set_width(120.0);
                            if let Some(ref id) = m.player1_id {
                                if let Some(tex) = app.logo_textures.get(id) {
                                    ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(48.0, 48.0)).corner_radius(8));
                                } else {
                                    ui.add_space(48.0);
                                }
                            }
                            ui.add_space(8.0);
                            ui.label(RichText::new(&p1).size(15.0).color(if w1 { theme::SUCCESS } else { theme::TEXT_PRIMARY }).strong());
                            if m.status == MatchStatus::Completed {
                                ui.label(RichText::new(m.score1.to_string()).size(24.0).color(if w1 { theme::SUCCESS } else { theme::TEXT_SECONDARY }).strong());
                            }
                        });

                        ui.vertical_centered(|ui| {
                            ui.set_width(40.0);
                            ui.add_space(20.0);
                            ui.label(RichText::new("VS").size(14.0).color(theme::TEXT_MUTED));
                        });

                        ui.vertical_centered(|ui| {
                            ui.set_width(120.0);
                            if let Some(ref id) = m.player2_id {
                                if let Some(tex) = app.logo_textures.get(id) {
                                    ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(48.0, 48.0)).corner_radius(8));
                                } else {
                                    ui.add_space(48.0);
                                }
                            }
                            ui.add_space(8.0);
                            ui.label(RichText::new(&p2).size(15.0).color(if w2 { theme::SUCCESS } else { theme::TEXT_PRIMARY }).strong());
                            if m.status == MatchStatus::Completed {
                                ui.label(RichText::new(m.score2.to_string()).size(24.0).color(if w2 { theme::SUCCESS } else { theme::TEXT_SECONDARY }).strong());
                            }
                        });
                    });

                    ui.add_space(24.0);

                    // Score Entry
                    if m.status == MatchStatus::InProgress {
                        ui.label(theme::section_header("Report Score"));
                        ui.add_space(8.0);
                        
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&p1).color(theme::TEXT_SECONDARY));
                                ui.add(egui::TextEdit::singleline(&mut app.score_input[0]).desired_width(130.0).hint_text("Score"));
                            });
                            ui.add_space(20.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&p2).color(theme::TEXT_SECONDARY));
                                ui.add(egui::TextEdit::singleline(&mut app.score_input[1]).desired_width(130.0).hint_text("Score"));
                            });
                        });
                        
                        ui.add_space(16.0);
                        
                        if ui.add(egui::Button::new(RichText::new("Submit Match Result").size(14.0).color(theme::BG_DARK).strong())
                            .fill(theme::ACCENT_BRONZE).corner_radius(theme::button_rounding())
                            .min_size(Vec2::new(ui.available_width(), 36.0))).clicked() {
                            app.submit_match_score();
                        }
                    }

                    if let Some((ref msg, ref mt)) = app.status_message {
                        ui.add_space(12.0);
                        let c = match mt {
                            crate::app::MessageType::Success => theme::SUCCESS,
                            crate::app::MessageType::Error => theme::ERROR,
                            crate::app::MessageType::Info => theme::INFO,
                        };
                        ui.label(RichText::new(msg).color(c).size(13.0));
                    }
                }
            }
        });

    app.show_match_modal = is_open;
}
