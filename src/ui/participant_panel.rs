use egui::{self, Ui, RichText, Vec2, Color32};

use crate::app::TourviaApp;
use crate::ui::theme;

pub fn render(app: &mut TourviaApp, ui: &mut Ui, ctx: &egui::Context) {
    let is_draft = app.is_draft();

    ui.horizontal(|ui| {
        ui.label(theme::subheading_text(&format!("Participants ({})", app.participants.len())));

        if is_draft {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new("🔀 Shuffle Seeds").color(theme::TEXT_PRIMARY())).fill(theme::BG_CARD())).clicked() {
                    app.auto_seed_participants();
                }

                if app.participants.len() >= 2 {
                    if ui.add(egui::Button::new(RichText::new("⚡ Generate Bracket").color(theme::BG_DARK()).strong()).fill(theme::ACCENT_BRONZE())).clicked() {
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
            .fill(theme::BG_PANEL())
            .stroke(theme::card_stroke())
            .corner_radius(theme::card_rounding())
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Add Participant:").color(theme::TEXT_SECONDARY()));
                    
                    let active_game = app.active_tournament.as_ref().map(|t| t.game_name.clone()).unwrap_or_default();
                    let filtered_rosters: Vec<_> = app.global_rosters.iter()
                        .filter(|r| r.game == active_game)
                        .cloned()
                        .collect();
                    
                    if filtered_rosters.is_empty() {
                        ui.label(RichText::new(format!("⚠️ No teams registered for '{}' in Global Roster.", active_game)).color(theme::WARNING()));
                    } else {
                        ui.menu_button(RichText::new("👥 Select From Roster").color(theme::TEXT_PRIMARY()), |ui| {
                            for r in filtered_rosters {
                                // Prevent adding the same team twice
                                let already_added = app.participants.iter().any(|p| p.name == r.name);
                                if already_added {
                                    ui.add_enabled(false, egui::Button::new(format!("{} (Already added)", r.name)));
                                } else {
                                    if ui.button(&r.name).clicked() {
                                        app.new_participant_name = r.name.clone();
                                        app.add_participant();
                                        ui.close_menu();
                                    }
                                }
                            }
                        });
                    }
                });
            });

        ui.add_space(16.0);
    }

    

    // ─── Participant List ──────────────────────────────
    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.add_space(24.0); // Space for drag handle
                ui.allocate_ui(Vec2::new(50.0, 20.0), |ui| ui.label(theme::label_text("Seed")));
                ui.allocate_ui(Vec2::new(50.0, 20.0), |ui| ui.label(theme::label_text("Logo")));
                ui.allocate_ui(Vec2::new(200.0, 20.0), |ui| ui.label(theme::label_text("Name")));
                if is_draft {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(theme::label_text("Actions"));
                    });
                }
            });
            
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let participants = app.participants.clone();
            let mut move_from_to: Option<(usize, usize)> = None;

            egui::ScrollArea::vertical().min_scrolled_height(0.0).show(ui, |ui| {
                for (idx, p) in participants.iter().enumerate() {
                    let item_id = egui::Id::new("participant_row").with(&p.id);
                    
                    let rect = ui.allocate_space(Vec2::new(ui.available_width(), 46.0)).1;
                    
                    let interact_sense = if is_draft { egui::Sense::drag() } else { egui::Sense::hover() };
                    let response = ui.interact(rect, item_id, interact_sense);
                    
                    if response.hovered() || response.dragged() {
                        ui.painter().rect_filled(rect, 4.0, theme::BG_CARD_HOVER());
                    } else {
                        ui.painter().rect_filled(rect, 4.0, theme::BG_CARD());
                    }
                    
                    // Draw drop indicator if hovering over another row while dragging
                    if is_draft && response.hovered() && ui.ctx().memory(|m| m.data.get_temp::<usize>(egui::Id::new("dragged_p")).is_some()) {
                        let dragged_idx = ui.ctx().memory(|m| m.data.get_temp::<usize>(egui::Id::new("dragged_p")).unwrap());
                        if dragged_idx != idx {
                            let y = if dragged_idx > idx { rect.top() } else { rect.bottom() };
                            ui.painter().hline(rect.x_range(), y, egui::Stroke::new(2.0, theme::ACCENT_BRONZE()));
                        }
                    }

                    // Content
                    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
                    child_ui.add_space(8.0);
                    
                    if is_draft {
                        child_ui.label(RichText::new("☰").color(theme::TEXT_MUTED()).size(16.0));
                    } else {
                        child_ui.add_space(16.0);
                    }
                    child_ui.add_space(8.0);

                    // Seed
                    child_ui.allocate_ui(Vec2::new(50.0, 46.0), |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new(format!("#{}", p.seed)).color(theme::TEXT_MUTED()).size(15.0));
                        });
                    });

                    // Logo
                    child_ui.allocate_ui(Vec2::new(50.0, 46.0), |ui| {
                        ui.centered_and_justified(|ui| {
                            if let Some(tex) = app.logo_textures.get(&p.id) {
                                ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(32.0, 32.0)).corner_radius(4));
                            } else {
                                ui.label(RichText::new("—").color(theme::TEXT_MUTED()));
                            }
                        });
                    });

                    // Name
                    child_ui.allocate_ui(Vec2::new(200.0, 46.0), |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label(RichText::new(&p.name).color(theme::TEXT_PRIMARY()).size(15.0).strong());
                        });
                    });

                    // Actions
                    if is_draft {
                        child_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(8.0);
                            if ui.add(egui::Button::new(RichText::new("🗑").color(theme::ERROR())).fill(Color32::TRANSPARENT)).clicked() {
                                app.delete_participant(idx);
                            }
                        });
                    }

                    // Handle Drag
                    if is_draft {
                        if response.drag_started() {
                            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("dragged_p"), idx));
                        }
                        if response.hovered() && ui.input(|i| i.pointer.any_released()) {
                            if let Some(dragged_idx) = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(egui::Id::new("dragged_p"))) {
                                if dragged_idx != idx {
                                    move_from_to = Some((dragged_idx, idx));
                                }
                            }
                        }
                    }
                }
            });

            if ui.input(|i| i.pointer.any_released()) {
                ui.ctx().memory_mut(|m| m.data.remove::<usize>(egui::Id::new("dragged_p")));
            }

            if let Some((from, to)) = move_from_to {
                app.move_participant_to(from, to);
            }
        });
}
