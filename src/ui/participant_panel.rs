use egui::{self, Ui, RichText, Vec2, Color32};

use crate::app::TourviaApp;
use crate::ui::theme;

pub fn render(app: &mut TourviaApp, ui: &mut Ui, ctx: &egui::Context) {
    let is_draft = app.is_draft();

    ui.horizontal(|ui| {
        ui.label(theme::subheading_text(&format!("Participants ({})", app.participants.len())));

        if is_draft {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new("🔀 Shuffle Seeds").color(theme::TEXT_PRIMARY)).fill(theme::BG_CARD)).clicked() {
                    app.auto_seed_participants();
                }

                if app.participants.len() >= 2 {
                    if ui.add(egui::Button::new(RichText::new("⚡ Generate Bracket").color(theme::BG_DARK).strong()).fill(theme::ACCENT_BRONZE)).clicked() {
                        app.generate_bracket();
                    }
                }
            });
        }
    });

    ui.add_space(16.0);

    // ─── Add Participant Form ───────────────────────────
    if is_draft {
        egui::Frame::new()
            .fill(theme::BG_PANEL)
            .stroke(theme::card_stroke())
            .corner_radius(theme::card_rounding())
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Add Participant:").color(theme::TEXT_SECONDARY));
                    let name_edit = egui::TextEdit::singleline(&mut app.new_participant_name)
                        .hint_text("Team / Player name")
                        .desired_width(200.0);
                    let resp = ui.add(name_edit);

                    if ui.add(egui::Button::new(RichText::new("Add").color(theme::BG_DARK)).fill(theme::SUCCESS)).clicked() 
                        || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        app.add_participant();
                    }

                    ui.add_space(16.0);
                    ui.label(RichText::new("or").color(theme::TEXT_MUTED));
                    ui.add_space(16.0);

                    let bulk_label = if app.show_bulk_add { "Cancel Bulk Add" } else { "Bulk Add..." };
                    if ui.add(egui::Button::new(RichText::new(bulk_label).color(theme::TEXT_PRIMARY)).fill(theme::BG_CARD)).clicked() {
                        app.show_bulk_add = !app.show_bulk_add;
                    }
                });

                if app.show_bulk_add {
                    ui.add_space(12.0);
                    ui.label(theme::small_text("Enter multiple names (one per line):"));
                    ui.add(egui::TextEdit::multiline(&mut app.bulk_add_text).desired_width(400.0).desired_rows(5));
                    ui.add_space(8.0);
                    if ui.add(egui::Button::new(RichText::new("Add All").color(theme::BG_DARK)).fill(theme::ACCENT_BRONZE)).clicked() {
                        app.bulk_add_participants();
                    }
                }
            });

        ui.add_space(16.0);
    }

    if let Some((ref msg, ref msg_type)) = app.status_message {
        let color = match msg_type {
            crate::app::MessageType::Success => theme::SUCCESS,
            crate::app::MessageType::Error => theme::ERROR,
            crate::app::MessageType::Info => theme::INFO,
        };
        ui.label(RichText::new(msg).color(color));
        ui.add_space(8.0);
    }

    // ─── Participant Table ──────────────────────────────
    egui::Frame::new()
        .fill(theme::BG_PANEL)
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            let mut table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::exact(60.0))          // Seed
                .column(egui_extras::Column::exact(50.0))          // Logo
                .column(egui_extras::Column::remainder().at_least(200.0)); // Name
                
            if is_draft {
                table = table.column(egui_extras::Column::exact(220.0)); // Actions
            }
            
            table = table.min_scrolled_height(0.0);

            table.header(30.0, |mut header| {
                header.col(|ui| { ui.label(theme::label_text("Seed")); });
                header.col(|ui| { ui.label(theme::label_text("Logo")); });
                header.col(|ui| { ui.label(theme::label_text("Participant Name")); });
                if is_draft {
                    header.col(|ui| { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(theme::label_text("Actions")); }); });
                }
            })
            .body(|mut body| {
                let participants = app.participants.clone();
                let count = participants.len();
                
                for (idx, p) in participants.iter().enumerate() {
                    body.row(42.0, |mut row| {
                        // Seed
                        row.col(|ui| {
                            ui.label(RichText::new(format!("#{}", p.seed)).color(theme::TEXT_MUTED).size(15.0));
                        });

                        // Logo
                        row.col(|ui| {
                            if let Some(tex) = app.logo_textures.get(&p.id) {
                                ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(32.0, 32.0)).corner_radius(4));
                            } else {
                                ui.label(RichText::new("—").color(theme::TEXT_MUTED));
                            }
                        });

                        // Name
                        row.col(|ui| {
                            ui.label(RichText::new(&p.name).color(theme::TEXT_PRIMARY).size(15.0).strong());
                        });

                        // Actions
                        if is_draft {
                            row.col(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new(RichText::new("🗑").color(theme::ERROR)).fill(Color32::TRANSPARENT)).clicked() {
                                        app.delete_participant(idx);
                                    }

                                    if ui.add(egui::Button::new("📷 Logo").fill(theme::BG_CARD)).clicked() {
                                        app.import_logo_for_participant(idx, ctx);
                                    }

                                    ui.horizontal(|ui| {
                                        let btn_size = Vec2::new(28.0, 24.0);
                                        if idx > 0 {
                                            if ui.add_sized(btn_size, egui::Button::new("▲").fill(theme::BG_CARD)).clicked() {
                                                app.move_participant_up(idx);
                                            }
                                        } else {
                                            ui.allocate_exact_size(btn_size, egui::Sense::hover());
                                        }

                                        if idx < count - 1 {
                                            if ui.add_sized(btn_size, egui::Button::new("▼").fill(theme::BG_CARD)).clicked() {
                                                app.move_participant_down(idx);
                                            }
                                        } else {
                                            ui.allocate_exact_size(btn_size, egui::Sense::hover());
                                        }
                                    });
                                });
                            });
                        }
                    });
                }
            });
        });
}
