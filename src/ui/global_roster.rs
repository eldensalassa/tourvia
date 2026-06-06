use eframe::egui::{self, Color32, RichText, Vec2};
use crate::app::TourviaApp;
use crate::ui::theme;
use crate::domain::roster::Roster;
use crate::domain::game::Game;

pub fn render(app: &mut TourviaApp, ui: &mut egui::Ui) {
    if app.active_roster.is_some() {
        render_detail(app, ui);
    } else {
        render_list(app, ui);
    }
}

fn render_list(app: &mut TourviaApp, ui: &mut egui::Ui) {
    egui::SidePanel::left("db_sidebar")
        .frame(egui::Frame::new().fill(theme::BG_PANEL()).inner_margin(egui::Margin::same(20)).stroke(egui::Stroke::new(1.0, theme::BORDER_SUBTLE())))
        .min_width(320.0)
        .max_width(320.0)
        .show_inside(ui, |ui| {
            // Manage Games section
            ui.label(theme::heading_text("🎮 Manage Games"));
            ui.add_space(16.0);
            
            ui.label(RichText::new("Game Name").color(theme::TEXT_MUTED()).size(13.0));
            ui.add_space(4.0);
            let name_edit = egui::TextEdit::singleline(&mut app.new_game_name).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(8, 6));
            let resp = ui.add(name_edit);
            
            ui.add_space(8.0);
            let btn = egui::Button::new(RichText::new("+ Add Game").color(theme::BG_DARK()).strong())
                .fill(theme::ACCENT_BRONZE())
                .corner_radius(theme::button_rounding())
                .min_size(Vec2::new(ui.available_width(), 36.0));
            
            if ui.add(btn).clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                let name = app.new_game_name.trim().to_string();
                if name.is_empty() {
                    app.notifications.error("Game name cannot be empty");
                } else if app.global_games.iter().any(|g| g.name.eq_ignore_ascii_case(&name)) {
                    app.notifications.error("Game already exists");
                } else {
                    let game = Game::new(name);
                    use crate::domain::repositories::GameRepository;
                    match app.db.create_game(&game) {
                        Ok(_) => {
                            app.notifications.success(format!("Added Game: {}", game.name));
                            app.new_game_name.clear();
                            app.load_games();
                        }
                        Err(e) => app.notifications.error(format!("Failed to add: {}", e)),
                    }
                }
            }
            
            ui.add_space(16.0);
            
            egui::ScrollArea::vertical().id_salt("games_scroll").max_height(200.0).show(ui, |ui| {
                for g in app.global_games.clone() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&g.name).color(theme::TEXT_PRIMARY()).size(14.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(RichText::new("✖").color(theme::ERROR())).fill(Color32::TRANSPARENT)).clicked() {
                                use crate::domain::repositories::GameRepository;
                                if let Err(e) = app.db.delete_game(&g.id) {
                                    app.notifications.error(format!("Delete failed: {}", e));
                                } else {
                                    app.notifications.info(format!("Deleted game {}", g.name));
                                    app.load_games();
                                }
                            }
                        });
                    });
                    ui.add_space(4.0);
                    ui.painter().hline(ui.min_rect().left()..=(ui.min_rect().right() + 20.0), ui.cursor().top(), egui::Stroke::new(1.0, theme::BORDER_SUBTLE()));
                    ui.add_space(4.0);
                }
            });

            ui.add_space(32.0);
            
            // Manage Teams section
            ui.label(theme::heading_text("⚔ Add Team / Player"));
            ui.add_space(16.0);
            
            if app.global_games.is_empty() {
                ui.label(RichText::new("Please add at least one Game above before adding teams.").color(theme::WARNING()));
            } else {
                if app.new_roster_game.is_empty() && !app.global_games.is_empty() {
                    app.new_roster_game = app.global_games[0].name.clone();
                }
                
                ui.label(RichText::new("Name").color(theme::TEXT_MUTED()).size(13.0));
                ui.add_space(4.0);
                let name_edit2 = egui::TextEdit::singleline(&mut app.new_roster_name).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(8, 6));
                let resp2 = ui.add(name_edit2);
                
                ui.add_space(12.0);
                
                ui.label(RichText::new("Game").color(theme::TEXT_MUTED()).size(13.0));
                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("roster_game_combo")
                    .selected_text(&app.new_roster_game)
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for g in &app.global_games {
                            ui.selectable_value(&mut app.new_roster_game, g.name.clone(), &g.name);
                        }
                    });
                
                ui.add_space(12.0);

                ui.label(RichText::new("Logo").color(theme::TEXT_MUTED()).size(13.0));
                ui.add_space(4.0);
                let mut open_dialog = false;
                
                if app.new_roster_logo.is_some() {
                    ui.horizontal(|ui| {
                        if let Some(texture) = app.logo_textures.get("__new_roster_logo") {
                            ui.add(egui::Image::new(texture).fit_to_exact_size(Vec2::new(36.0, 36.0)).corner_radius(18.0));
                        }
                        if ui.add(egui::Button::new(RichText::new("🖼 Change").color(theme::TEXT_PRIMARY()))
                            .fill(theme::BG_CARD())
                            .corner_radius(theme::button_rounding())
                            .min_size(Vec2::new(80.0, 32.0))
                        ).clicked() {
                            open_dialog = true;
                        }
                        if ui.add(egui::Button::new(RichText::new("❌").color(theme::ERROR()))
                            .fill(theme::BG_CARD())
                            .corner_radius(theme::button_rounding())
                            .min_size(Vec2::new(32.0, 32.0))
                        ).clicked() {
                            app.new_roster_logo = None;
                            app.logo_textures.remove("__new_roster_logo");
                        }
                    });
                } else {
                    if ui.add(egui::Button::new(RichText::new("🖼 Select Logo").color(theme::TEXT_PRIMARY()))
                        .fill(theme::BG_CARD())
                        .corner_radius(theme::button_rounding())
                        .min_size(Vec2::new(ui.available_width(), 36.0))
                    ).clicked() {
                        open_dialog = true;
                    }
                }

                if open_dialog {
                    app.image_picker_open = true;
                    app.image_picker_target = Some(crate::app::ImageTarget::NewRosterLogo);
                    app.image_picker_query = app.new_roster_name.clone();
                }
                
                ui.add_space(20.0);
                
                let btn2 = egui::Button::new(RichText::new("+ Add Team").color(theme::BG_DARK()).strong())
                    .fill(theme::ACCENT_BRONZE())
                    .corner_radius(theme::button_rounding())
                    .min_size(Vec2::new(ui.available_width(), 36.0));
                    
                if ui.add(btn2).clicked() || (resp2.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    let name = app.new_roster_name.trim().to_string();
                    let game = app.new_roster_game.trim().to_string();
                    
                    if name.is_empty() {
                        app.notifications.error("Name cannot be empty");
                    } else if game.is_empty() {
                        app.notifications.error("Game must be selected");
                    } else {
                        let roster = Roster::new(name, game, app.new_roster_logo.clone());
                        use crate::domain::repositories::RosterRepository;
                        match app.db.create_roster(&roster) {
                            Ok(_) => {
                                app.notifications.success(format!("Added {} to {}", roster.name, roster.game));
                                app.new_roster_name.clear();
                                app.new_roster_logo = None;
                                app.load_rosters();
                            }
                            Err(e) => {
                                app.notifications.error(format!("Failed to add: {}", e));
                            }
                        }
                    }
                }
            }
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(theme::BG_DARK()).inner_margin(egui::Margin::same(24)))
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(theme::heading_text("👥 Database Repository").size(24.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(
                        egui::Button::new(RichText::new("⬅ Back to Dashboard").color(theme::TEXT_PRIMARY()))
                            .fill(theme::BG_CARD_HOVER())
                            .corner_radius(theme::button_rounding())
                            .min_size(Vec2::new(140.0, 36.0))
                    ).clicked() {
                        app.go_to_dashboard();
                    }
                });
            });
            
            ui.add_space(24.0);
            
            egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("roster_grid_scroll").show(ui, |ui| {
                let rosters = app.global_rosters.clone();
                let mut is_clicked_idx = None;

                if rosters.is_empty() {
                    ui.label(RichText::new("No teams in the database yet. Add one from the sidebar!").color(theme::TEXT_MUTED()).italics());
                } else {
                    let mut grouped_rosters: std::collections::BTreeMap<String, Vec<(usize, crate::domain::roster::Roster)>> = std::collections::BTreeMap::new();
                    for (idx, r) in rosters.into_iter().enumerate() {
                        grouped_rosters.entry(r.game.clone()).or_default().push((idx, r));
                    }

                    for (game_name, game_rosters) in grouped_rosters {
                        ui.label(RichText::new(&game_name).color(theme::TEXT_PRIMARY()).size(18.0).strong());
                        ui.add_space(12.0);

                        ui.scope(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(20.0, 20.0);
                            ui.horizontal_wrapped(|ui| {
                                for (idx, r) in game_rosters {
                                    // Pre-load logo if needed
                                    if let Some(ref logo_data) = r.logo_data {
                                        if !app.logo_textures.contains_key(&r.id) {
                                            if let Some(texture) = crate::app::TourviaApp::decode_logo(ui.ctx(), &r.id, logo_data) {
                                                app.logo_textures.insert(r.id.clone(), texture);
                                            }
                                        }
                                    }

                                    let card_size = Vec2::new(160.0, 180.0);
                                    ui.allocate_ui(card_size, |ui| {
                                        let (rect, response) = ui.allocate_exact_size(card_size, egui::Sense::click());
                                        
                                        let is_hovered = response.hovered();
                                        let bg_color = if is_hovered { theme::BG_CARD_HOVER() } else { theme::BG_ELEVATED() };
                                        
                                        ui.painter().rect_filled(rect, 12.0, bg_color);
                                        ui.painter().rect_stroke(rect, 12.0, egui::Stroke::new(1.0, theme::BORDER_SUBTLE()), egui::StrokeKind::Inside);
                                        
                                        if is_hovered {
                                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                                        }
                                        if response.clicked() {
                                            is_clicked_idx = Some(idx);
                                        }

                                        // Logo
                                        let logo_rect = egui::Rect::from_center_size(
                                            rect.center() - Vec2::new(0.0, 20.0),
                                            Vec2::new(72.0, 72.0),
                                        );
                                        
                                        if let Some(texture) = app.logo_textures.get(&r.id) {
                                            let img = egui::Image::new(texture).fit_to_exact_size(Vec2::new(72.0, 72.0)).corner_radius(36.0);
                                            ui.put(logo_rect, img);
                                        } else {
                                            ui.painter().circle_filled(logo_rect.center(), 36.0, theme::BG_DARK());
                                            ui.painter().text(logo_rect.center(), egui::Align2::CENTER_CENTER, "🛡", egui::FontId::proportional(32.0), theme::TEXT_MUTED());
                                        }
                                        
                                        // Text
                                        let text_rect = egui::Rect::from_min_max(
                                            egui::Pos2::new(rect.left() + 8.0, logo_rect.bottom() + 16.0),
                                            egui::Pos2::new(rect.right() - 8.0, rect.bottom() - 8.0),
                                        );
                                        
                                        ui.painter().text(
                                            text_rect.center_top(),
                                            egui::Align2::CENTER_TOP,
                                            &r.name,
                                            egui::FontId::proportional(15.0),
                                            theme::TEXT_PRIMARY(),
                                        );
                                        
                                        ui.painter().text(
                                            text_rect.center_top() + Vec2::new(0.0, 20.0),
                                            egui::Align2::CENTER_TOP,
                                            &r.game,
                                            egui::FontId::proportional(12.0),
                                            theme::TEXT_MUTED(),
                                        );

                                        // Delete button (custom drawing for pixel perfect position)
                                        let delete_rect = egui::Rect::from_min_size(
                                            egui::Pos2::new(rect.right() - 32.0, rect.top() + 8.0),
                                            Vec2::new(24.0, 24.0)
                                        );
                                        let del_resp = ui.interact(delete_rect, egui::Id::new(format!("del_{}", r.id)), egui::Sense::click());
                                        let del_color = if del_resp.hovered() { theme::ERROR() } else { theme::TEXT_MUTED() };
                                        let del_bg = if del_resp.hovered() { theme::BG_PANEL() } else { Color32::TRANSPARENT };
                                        
                                        ui.painter().rect_filled(delete_rect, 6.0, del_bg);
                                        ui.painter().text(delete_rect.center(), egui::Align2::CENTER_CENTER, "🗑", egui::FontId::proportional(14.0), del_color);

                                        if del_resp.clicked() {
                                            use crate::domain::repositories::RosterRepository;
                                            if let Err(e) = app.db.delete_roster(&r.id) {
                                                app.notifications.error(format!("Delete failed: {}", e));
                                            } else {
                                                app.notifications.info(format!("Deleted {}", r.name));
                                                app.load_rosters();
                                            }
                                        }
                                    });
                                }
                            });
                        });
                        
                        ui.add_space(24.0); // Spacing between games
                    }
                }

                if let Some(idx) = is_clicked_idx {
                    app.open_roster(idx);
                }
            });
        });
}

fn render_detail(app: &mut TourviaApp, ui: &mut egui::Ui) {
    let roster = if let Some(r) = app.active_roster.clone() {
        r
    } else {
        return;
    };
    
    // --- SIDE PANEL ---
    egui::SidePanel::left("detail_sidebar")
        .frame(egui::Frame::new().fill(theme::BG_PANEL()).inner_margin(egui::Margin::same(20)).stroke(egui::Stroke::new(1.0, theme::BORDER_SUBTLE())))
        .min_width(320.0)
        .max_width(320.0)
        .show_inside(ui, |ui| {
            // BACK BUTTON
            if ui.add(egui::Button::new(RichText::new("⬅ Back to Database").color(theme::TEXT_PRIMARY()))
                .fill(theme::BG_CARD_HOVER())
                .corner_radius(theme::button_rounding())
            ).clicked() {
                app.close_roster();
            }
            ui.add_space(24.0);

            // TEAM IDENTITY (CENTERED)
            ui.vertical_centered(|ui| {
                // Logo
                let logo_size = Vec2::new(100.0, 100.0);
                let (rect, _) = ui.allocate_exact_size(logo_size, egui::Sense::hover());
                if let Some(texture) = app.logo_textures.get(&roster.id) {
                    ui.put(rect, egui::Image::new(texture).fit_to_exact_size(logo_size).corner_radius(50.0));
                } else {
                    ui.painter().circle_filled(rect.center(), 50.0, theme::BG_DARK());
                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "🛡", egui::FontId::proportional(48.0), theme::TEXT_MUTED());
                }
                
                ui.add_space(12.0);
                ui.label(theme::heading_text(&roster.name).size(22.0));
                ui.label(RichText::new(&roster.game).color(theme::TEXT_MUTED()).size(14.0));
                
                ui.add_space(12.0);
                if ui.add(egui::Button::new(RichText::new("🖼 Change Logo").color(theme::TEXT_PRIMARY()))
                    .fill(theme::BG_ELEVATED())
                    .corner_radius(theme::button_rounding())
                ).clicked() {
                    app.image_picker_open = true;
                    app.image_picker_target = Some(crate::app::ImageTarget::ExistingRosterLogo(roster.id.clone()));
                    app.image_picker_query = roster.name.clone();
                }
            });
            
            ui.add_space(24.0);
            ui.separator();
            ui.add_space(16.0);

            // DESCRIPTION EDITOR
            let editing_id = egui::Id::new("editing_roster_desc");
            let is_editing = ui.ctx().memory(|m| m.data.get_temp::<bool>(editing_id).unwrap_or(false));
            let mut desc_changed = false;
            let mut current_desc = roster.description.clone();

            ui.horizontal(|ui| {
                ui.label(theme::subheading_text("Team Description").size(16.0));
                if !is_editing {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new("✏ Edit").color(theme::TEXT_PRIMARY())).fill(theme::BG_ELEVATED()).corner_radius(theme::button_rounding())).clicked() {
                            ui.ctx().memory_mut(|m| m.data.insert_temp(editing_id, true));
                        }
                    });
                }
            });
            ui.add_space(8.0);
            
            if is_editing {
                let desc_edit = egui::TextEdit::multiline(&mut current_desc)
                    .desired_width(f32::INFINITY)
                    .min_size(Vec2::new(0.0, 80.0))
                    .hint_text("Enter team description...");
                let resp = ui.add(desc_edit);
                
                if resp.changed() {
                    if let Some(r) = app.active_roster.as_mut() {
                        r.description = current_desc.clone();
                    }
                }
                
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("💾 Save").color(theme::BG_DARK()).strong())
                        .fill(theme::SUCCESS())
                        .corner_radius(theme::button_rounding())
                    ).clicked() {
                        desc_changed = true;
                        ui.ctx().memory_mut(|m| m.data.insert_temp(editing_id, false));
                    }
                    if ui.add(egui::Button::new(RichText::new("✖ Cancel").color(theme::BG_DARK()).strong())
                        .fill(theme::ERROR())
                        .corner_radius(theme::button_rounding())
                    ).clicked() {
                        let r_id = app.active_roster.as_ref().map(|r| r.id.clone());
                        if let Some(id) = r_id {
                            app.load_rosters();
                            if let Some(reloaded) = app.global_rosters.iter().find(|x| x.id == id) {
                                app.active_roster = Some(reloaded.clone());
                            }
                        }
                        ui.ctx().memory_mut(|m| m.data.insert_temp(editing_id, false));
                    }
                });
            } else {
                egui::Frame::new().fill(theme::BG_DARK()).inner_margin(12).corner_radius(8).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if current_desc.trim().is_empty() {
                        ui.label(egui::RichText::new("No description provided.").color(theme::TEXT_MUTED()).italics());
                    } else {
                        ui.label(egui::RichText::new(&current_desc).color(theme::TEXT_PRIMARY()));
                    }
                });
            }

            if desc_changed {
                if let Some(r) = app.active_roster.as_ref() {
                    use crate::domain::repositories::RosterRepository;
                    if let Err(e) = app.db.update_roster(r) {
                        app.notifications.error(format!("Failed to save description: {}", e));
                    } else {
                        app.load_rosters();
                        app.notifications.success("Team description saved.");
                    }
                }
            }
            
            ui.add_space(24.0);
            ui.separator();
            ui.add_space(16.0);

            // ADD MEMBER
            ui.label(theme::subheading_text("Add Member").size(16.0));
            ui.add_space(8.0);
            ui.label(RichText::new("Name").color(theme::TEXT_MUTED()).size(13.0));
            ui.add_space(4.0);
            let name_edit = egui::TextEdit::singleline(&mut app.new_member_name).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(8, 6));
            let resp = ui.add(name_edit);
            
            ui.add_space(12.0);
            ui.label(RichText::new("Photo").color(theme::TEXT_MUTED()).size(13.0));
            ui.add_space(4.0);
            
            let mut open_member_dialog = false;
            ui.horizontal(|ui| {
                if app.new_member_photo.is_some() {
                    if let Some(texture) = app.member_photo_textures.get("__new_member_photo") {
                        ui.add(egui::Image::new(texture).fit_to_exact_size(Vec2::new(32.0, 32.0)).corner_radius(8.0));
                    }
                    if ui.add(egui::Button::new(RichText::new("📷 Change").color(theme::TEXT_PRIMARY()))
                        .fill(theme::BG_ELEVATED())
                        .corner_radius(theme::button_rounding())
                    ).clicked() {
                        open_member_dialog = true;
                    }
                    if ui.add(egui::Button::new("❌").fill(Color32::TRANSPARENT)).clicked() {
                        app.new_member_photo = None;
                        app.member_photo_textures.remove("__new_member_photo");
                    }
                } else {
                    if ui.add(egui::Button::new(RichText::new("📷 Select Photo").color(theme::TEXT_PRIMARY()))
                        .fill(theme::BG_ELEVATED())
                        .corner_radius(theme::button_rounding())
                    ).clicked() {
                        open_member_dialog = true;
                    }
                }
            });

            if open_member_dialog {
                app.image_picker_open = true;
                app.image_picker_target = Some(crate::app::ImageTarget::NewMemberPhoto);
                app.image_picker_query = app.new_member_name.clone();
            }
            
            ui.add_space(20.0);
            
            let btn = egui::Button::new(RichText::new("+ Add Member").color(theme::BG_DARK()).strong())
                .fill(theme::ACCENT_BRONZE())
                .corner_radius(theme::button_rounding());
                
            if ui.add_sized([ui.available_width(), 36.0], btn).clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                let name = app.new_member_name.trim().to_string();
                if name.is_empty() {
                    app.notifications.error("Member name cannot be empty");
                } else {
                    let member = crate::domain::roster::RosterMember::new(roster.id.clone(), name, app.new_member_photo.clone());
                    use crate::domain::repositories::RosterRepository;
                    match app.db.add_roster_member(&member) {
                        Ok(_) => {
                            app.notifications.success(format!("Added player {}", member.name));
                            app.new_member_name.clear();
                            app.new_member_photo = None;
                            app.load_active_roster_members();
                        }
                        Err(e) => app.notifications.error(format!("Failed to add member: {}", e)),
                    }
                }
            }
        });

    // --- CENTRAL PANEL ---
    app.ensure_member_photos_loaded(ui.ctx());

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(theme::BG_DARK()).inner_margin(egui::Margin::same(24)))
        .show_inside(ui, |ui| {
            ui.label(theme::heading_text("Roster Members").size(24.0));
            ui.add_space(20.0);

            if app.roster_members.is_empty() {
                ui.label(RichText::new("No members found. Add some from the sidebar!").color(theme::TEXT_MUTED()).italics());
            } else {
                egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("members_scroll").show(ui, |ui| {
                    let mut to_delete = None;

                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(20.0, 20.0);
                        ui.horizontal_wrapped(|ui| {
                            for m in &app.roster_members {
                                let card_size = Vec2::new(120.0, 160.0);
                                ui.allocate_ui(card_size, |ui| {
                                    let (rect, _response) = ui.allocate_exact_size(card_size, egui::Sense::hover());
                                    
                                    // Draw background
                                    ui.painter().rect_filled(rect, 12.0, theme::BG_ELEVATED());
                                    ui.painter().rect_stroke(rect, 12.0, egui::Stroke::new(1.0, theme::BORDER_SUBTLE()), egui::StrokeKind::Inside);
                                    
                                    // Photo (large)
                                    let photo_rect = egui::Rect::from_min_size(
                                        rect.min + Vec2::new(10.0, 10.0),
                                        Vec2::new(100.0, 100.0),
                                    );
                                    
                                    if m.profile_picture.is_some() {
                                        if let Some(tex) = app.member_photo_textures.get(&m.id) {
                                            let img = egui::Image::new(tex)
                                                .fit_to_exact_size(Vec2::new(100.0, 100.0))
                                                .corner_radius(10.0);
                                            ui.put(photo_rect, img);
                                        }
                                    } else {
                                        ui.painter().rect_filled(photo_rect, 10.0, theme::BG_DARK());
                                        ui.painter().text(
                                            photo_rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            "👤",
                                            egui::FontId::proportional(36.0),
                                            theme::TEXT_MUTED(),
                                        );
                                    }
                                    
                                    // Photo interaction (click to edit)
                                    let photo_response = ui.interact(photo_rect, egui::Id::new(&m.id).with("photo"), egui::Sense::click());
                                    if photo_response.hovered() {
                                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                                        ui.painter().rect_filled(photo_rect, 10.0, Color32::from_black_alpha(160));
                                        ui.painter().text(
                                            photo_rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            "📷 Edit",
                                            egui::FontId::proportional(14.0),
                                            Color32::WHITE,
                                        );
                                    }
                                    if photo_response.clicked() {
                                        app.image_picker_open = true;
                                        app.image_picker_target = Some(crate::app::ImageTarget::ExistingMemberPhoto(m.id.clone()));
                                        app.image_picker_query = m.name.clone();
                                    }
                                    
                                    // Name
                                    let name_rect = egui::Rect::from_min_max(
                                        egui::Pos2::new(rect.left() + 8.0, photo_rect.bottom() + 12.0),
                                        egui::Pos2::new(rect.right() - 8.0, rect.bottom() - 8.0),
                                    );
                                    ui.painter().text(
                                        name_rect.center_top(),
                                        egui::Align2::CENTER_TOP,
                                        &m.name,
                                        egui::FontId::proportional(14.0),
                                        theme::TEXT_PRIMARY(),
                                    );
                                    
                                    // Elegant delete button (custom drawn, top right)
                                    let delete_rect = egui::Rect::from_min_size(
                                        egui::Pos2::new(rect.right() - 28.0, rect.top() + 4.0),
                                        Vec2::new(24.0, 24.0)
                                    );
                                    let del_resp = ui.interact(delete_rect, egui::Id::new(format!("del_{}", m.id)), egui::Sense::click());
                                    let del_color = if del_resp.hovered() { theme::ERROR() } else { theme::TEXT_MUTED() };
                                    let del_bg = if del_resp.hovered() { theme::BG_PANEL() } else { Color32::from_black_alpha(150) };
                                    
                                    ui.painter().rect_filled(delete_rect, 6.0, del_bg);
                                    ui.painter().text(delete_rect.center(), egui::Align2::CENTER_CENTER, "✖", egui::FontId::proportional(14.0), del_color);

                                    if del_resp.clicked() {
                                        to_delete = Some(m.id.clone());
                                    }
                                });
                            }
                        });
                    });

                    if let Some(id) = to_delete {
                        use crate::domain::repositories::RosterRepository;
                        if let Err(e) = app.db.delete_roster_member(&id) {
                            app.notifications.error(format!("Failed to delete member: {}", e));
                        } else {
                            app.notifications.success("Member removed");
                            app.load_active_roster_members();
                        }
                    }
                });
            }
        });
}
