use egui::{self, Ui, RichText, Vec2, Color32};

use crate::app::TourviaApp;
use crate::ui::theme;
use crate::domain::repositories::RosterRepository;

pub fn render(app: &mut TourviaApp, ui: &mut Ui, _ctx: &egui::Context) {
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
                            ui.set_min_width(200.0); // Prevent squishing
                            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                                for r in filtered_rosters {
                                    // Prevent adding the same team twice
                                    let already_added = app.participants.iter().any(|p| p.name == r.name);
                                    if already_added {
                                        ui.add_enabled(false, egui::Button::new(format!("{} (Already added)", r.name)));
                                    } else {
                                        if ui.add(egui::Button::new(&r.name)).clicked() {
                                            app.new_participant_name = r.name.clone();
                                            app.add_participant();
                                            ui.close_menu();
                                        }
                                    }
                                }
                            });
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
                ui.add_space(8.0);
                if is_draft {
                    ui.add_space(16.0);
                } else {
                    ui.add_space(16.0);
                }
                ui.add_space(8.0);

                ui.allocate_ui(Vec2::new(50.0, 20.0), |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(theme::label_text("Seed"));
                    });
                });
                ui.allocate_ui(Vec2::new(50.0, 20.0), |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(theme::label_text("Logo"));
                    });
                });
                ui.allocate_ui(Vec2::new(200.0, 20.0), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(theme::label_text("Name"));
                    });
                });
                
                if is_draft {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);
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
                    let is_dragged = ui.ctx().memory(|m| m.data.get_temp::<usize>(egui::Id::new("dragged_p"))) == Some(idx);
                    
                    let rect = ui.allocate_space(Vec2::new(ui.available_width(), 46.0)).1;
                    
                    let interact_sense = egui::Sense::click();
                    let response = ui.interact(rect, item_id, interact_sense);
                    
                    if response.clicked() {
                        if let Some(r) = app.global_rosters.iter().find(|r| r.name == p.name) {
                            app.participant_preview_roster = Some(r.clone());
                            if let Ok(members) = app.db.get_roster_members(&r.id) {
                                app.participant_preview_members = members;
                            }
                        }
                    }
                    
                    if is_dragged {
                        ui.painter().rect_filled(rect, 4.0, theme::BG_PANEL());
                        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, theme::TEXT_MUTED().linear_multiply(0.2)), egui::StrokeKind::Inside);
                    } else {
                        if response.hovered() {
                            ui.painter().rect_filled(rect, 4.0, theme::BG_CARD_HOVER());
                        } else {
                            ui.painter().rect_filled(rect, 4.0, theme::BG_CARD());
                        }
                    }
                    
                    let mut is_pointer_over = false;
                    if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                        is_pointer_over = rect.contains(pointer_pos);
                    }
                    
                    // Draw drop indicator if hovering over another row while dragging
                    if is_draft && is_pointer_over && ui.ctx().memory(|m| m.data.get_temp::<usize>(egui::Id::new("dragged_p")).is_some()) {
                        let dragged_idx = ui.ctx().memory(|m| m.data.get_temp::<usize>(egui::Id::new("dragged_p")).unwrap());
                        if dragged_idx != idx {
                            let y = if dragged_idx > idx { rect.top() } else { rect.bottom() };
                            ui.painter().hline(rect.x_range(), y, egui::Stroke::new(2.0, theme::ACCENT_BRONZE()));
                        }
                    }

                    if is_dragged {
                        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                            let mut float_pos = pointer_pos - Vec2::new(16.0, 23.0);
                            float_pos.x = rect.min.x; // Lock X to row position
                            
                            let clip_rect = ui.clip_rect();
                            float_pos.y = float_pos.y.clamp(clip_rect.min.y, clip_rect.max.y - rect.height());
                            
                            let float_rect = egui::Rect::from_min_size(float_pos, rect.size());
                            egui::Area::new(item_id.with("floating"))
                                .fixed_pos(float_rect.min)
                                .interactable(false)
                                .order(egui::Order::Tooltip)
                                .show(ui.ctx(), |ui| {
                                    let (float_inner_rect, _) = ui.allocate_exact_size(rect.size(), egui::Sense::hover());
                                    ui.painter().rect_filled(float_inner_rect, 4.0, theme::BG_CARD_HOVER());
                                    ui.painter().rect_stroke(float_inner_rect, 4.0, egui::Stroke::new(1.0, theme::ACCENT_BRONZE()), egui::StrokeKind::Inside);
                                    
                                    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(float_inner_rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
                                    child_ui.add_space(8.0);
                                    
                                    let (handle_rect, _) = child_ui.allocate_exact_size(Vec2::new(16.0, 46.0), egui::Sense::hover());
                                    child_ui.painter().text(
                                        handle_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "☰",
                                        egui::FontId::proportional(16.0),
                                        theme::TEXT_MUTED(),
                                    );
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
                                                let size = tex.size_vec2();
                                                let aspect = if size.y > 0.0 { size.x / size.y } else { 1.0 };
                                                let w = if aspect > 1.0 { 32.0 } else { 32.0 * aspect };
                                                let h = if aspect > 1.0 { 32.0 / aspect } else { 32.0 };
                                                ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(w, h)).corner_radius(4));
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
                                });
                        }
                    } else {
                        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
                        child_ui.add_space(8.0);
                        
                        if is_draft {
                            let (handle_rect, handle_response) = child_ui.allocate_exact_size(Vec2::new(16.0, 46.0), egui::Sense::drag());
                            child_ui.painter().text(
                                handle_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "☰",
                                egui::FontId::proportional(16.0),
                                if handle_response.hovered() { theme::TEXT_PRIMARY() } else { theme::TEXT_MUTED() },
                            );
                            
                            if handle_response.drag_started() {
                                ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("dragged_p"), idx));
                            }
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
                                    let size = tex.size_vec2();
                                    let aspect = if size.y > 0.0 { size.x / size.y } else { 1.0 };
                                    let w = if aspect > 1.0 { 32.0 } else { 32.0 * aspect };
                                    let h = if aspect > 1.0 { 32.0 / aspect } else { 32.0 };
                                    ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(w, h)).corner_radius(4));
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
                    }

                    // Handle Drag Drop Release
                    if is_draft {
                        // The drag_started is now handled by the handle_response above.
                        if ui.input(|i| i.pointer.any_released()) {
                            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                if rect.contains(pointer_pos) {
                                    if let Some(dragged_idx) = ui.ctx().memory_mut(|m| m.data.get_temp::<usize>(egui::Id::new("dragged_p"))) {
                                        if dragged_idx != idx {
                                            move_from_to = Some((dragged_idx, idx));
                                        }
                                    }
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

pub fn render_preview_modal(app: &mut TourviaApp, ctx: &egui::Context) {
    if app.participant_preview_roster.is_some() {
        let mut close_requested = false;
        
        egui::Window::new("Team Preview")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(egui::Frame::window(&ctx.style()).fill(theme::BG_PANEL()).stroke(theme::card_stroke()).inner_margin(egui::Margin::same(24)))
            .show(ctx, |ui| {
                if let Some(roster) = &app.participant_preview_roster {
                    ui.set_min_width(400.0);
                    
                    ui.vertical_centered(|ui| {
                        if let Some(tex) = app.logo_textures.get(&roster.id) {
                            let size = tex.size_vec2();
                            let aspect = if size.y > 0.0 { size.x / size.y } else { 1.0 };
                            let w = if aspect > 1.0 { 80.0 } else { 80.0 * aspect };
                            let h = if aspect > 1.0 { 80.0 / aspect } else { 80.0 };
                            ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(w, h)).corner_radius(8));
                        }
                        ui.add_space(8.0);
                        ui.label(RichText::new(&roster.name).font(egui::FontId::proportional(24.0)).strong().color(theme::TEXT_PRIMARY()));
                        if !roster.description.is_empty() {
                            ui.label(RichText::new(&roster.description).color(theme::TEXT_MUTED()));
                        }
                    });
                    
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    ui.label(theme::subheading_text("Team Members"));
                    ui.add_space(8.0);
                    
                    if app.participant_preview_members.is_empty() {
                        ui.label(RichText::new("No members found.").color(theme::TEXT_MUTED()).italics());
                    } else {
                        // Load missing textures
                        let mut new_textures = Vec::new();
                        for m in &app.participant_preview_members {
                            if m.profile_picture.is_some() && !app.member_photo_textures.contains_key(&m.id) {
                                if let Some(photo) = &m.profile_picture {
                                    if let Ok(image) = image::load_from_memory(photo) {
                                        let size = [image.width() as _, image.height() as _];
                                        let image_buffer = image.to_rgba8();
                                        let pixels = image_buffer.as_flat_samples();
                                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                                        let tex = ctx.load_texture(&m.id, color_image, egui::TextureOptions::LINEAR);
                                        new_textures.push((m.id.clone(), tex));
                                    }
                                }
                            }
                        }
                        for (id, tex) in new_textures {
                            app.member_photo_textures.insert(id, tex);
                        }

                        egui::ScrollArea::vertical().max_height(250.0).auto_shrink([false, true]).show(ui, |ui| {
                            for member in &app.participant_preview_members {
                                egui::Frame::new()
                                    .fill(theme::BG_CARD())
                                    .corner_radius(theme::card_rounding())
                                    .inner_margin(egui::Margin::same(12))
                                    .show(ui, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            if let Some(tex) = app.member_photo_textures.get(&member.id) {
                                                let size = tex.size_vec2();
                                                let aspect = if size.y > 0.0 { size.x / size.y } else { 1.0 };
                                                let w = if aspect > 1.0 { 32.0 } else { 32.0 * aspect };
                                                let h = if aspect > 1.0 { 32.0 / aspect } else { 32.0 };
                                                ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(w, h)).corner_radius(4));
                                            } else {
                                                let (rect, _) = ui.allocate_exact_size(Vec2::new(32.0, 32.0), egui::Sense::hover());
                                                ui.painter().rect_filled(rect, 4.0, theme::BG_CARD_HOVER());
                                                ui.painter().text(
                                                    rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    "👤",
                                                    egui::FontId::proportional(16.0),
                                                    theme::TEXT_MUTED(),
                                                );
                                            }
                                            ui.add_space(12.0);
                                            ui.vertical(|ui| {
                                                ui.label(RichText::new(&member.name).strong().size(16.0).color(theme::TEXT_PRIMARY()));
                                            });
                                        });
                                    });
                                ui.add_space(8.0);
                            }
                        });
                    }
                    
                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        if ui.add(egui::Button::new(RichText::new("Close").color(theme::TEXT_PRIMARY())).fill(theme::BG_CARD()).min_size(Vec2::new(100.0, 30.0))).clicked() {
                            close_requested = true;
                        }
                    });
                }
            });
            
        if close_requested {
            app.participant_preview_roster = None;
            app.participant_preview_members.clear();
        }
    }
}
