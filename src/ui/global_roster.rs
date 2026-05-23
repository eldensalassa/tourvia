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
    ui.horizontal(|ui| {
        ui.label(theme::heading_text("👥 Database"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(
                egui::Button::new(RichText::new("⬅ Back to Dashboard").color(theme::TEXT_PRIMARY()))
                    .fill(theme::BG_CARD_HOVER())
                    .corner_radius(theme::button_rounding())
            ).clicked() {
                app.go_to_dashboard();
            }
        });
    });
    
    ui.add_space(20.0);
    
    // Manage Games section
    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.label(theme::subheading_text("Manage Games"));
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("Game Name:").color(theme::TEXT_MUTED()));
                let name_edit = egui::TextEdit::singleline(&mut app.new_game_name).desired_width(200.0);
                let resp = ui.add(name_edit);
                
                ui.add_space(16.0);
                
                let btn = egui::Button::new(RichText::new("+ Add Game").color(theme::BG_DARK()).strong())
                    .fill(theme::ACCENT_BRONZE())
                    .corner_radius(theme::button_rounding())
                    .min_size(Vec2::new(100.0, 32.0));
                
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
            });
            
            ui.add_space(12.0);
            
            egui::ScrollArea::vertical().id_salt("games_scroll").max_height(150.0).show(ui, |ui| {
                for g in app.global_games.clone() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&g.name).color(theme::TEXT_PRIMARY()).strong().size(14.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(RichText::new("🗑").color(theme::ERROR())).fill(Color32::TRANSPARENT)).clicked() {
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
                    ui.separator();
                }
            });
        });
        
    ui.add_space(20.0);
    
    // Manage Teams section
    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.label(theme::subheading_text("Manage Teams/Players"));
            ui.add_space(8.0);
            
            if app.global_games.is_empty() {
                ui.label(RichText::new("Please add at least one Game above before adding teams.").color(theme::WARNING()));
            } else {
                if app.new_roster_game.is_empty() && !app.global_games.is_empty() {
                    app.new_roster_game = app.global_games[0].name.clone();
                }
                
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Name:").color(theme::TEXT_MUTED()));
                    let name_edit = egui::TextEdit::singleline(&mut app.new_roster_name).desired_width(180.0);
                    let resp = ui.add(name_edit);
                    
                    ui.add_space(16.0);
                    
                    ui.label(RichText::new("Game:").color(theme::TEXT_MUTED()));
                    egui::ComboBox::from_id_salt("roster_game_combo")
                        .selected_text(&app.new_roster_game)
                        .show_ui(ui, |ui| {
                            for g in &app.global_games {
                                ui.selectable_value(&mut app.new_roster_game, g.name.clone(), &g.name);
                            }
                        });
                    
                    ui.add_space(16.0);

                    // Logo Selection
                    let mut open_dialog = false;
                    
                    if app.new_roster_logo.is_some() {
                        ui.horizontal(|ui| {
                            if let Some(texture) = app.logo_textures.get("__new_roster_logo") {
                                ui.add(egui::Image::new(texture).fit_to_exact_size(Vec2::new(32.0, 32.0)).corner_radius(8.0));
                            }
                            if ui.add(egui::Button::new(RichText::new("🖼 Change").color(theme::TEXT_PRIMARY()))
                                .fill(theme::BG_CARD())
                                .corner_radius(theme::button_rounding())
                            ).clicked() {
                                open_dialog = true;
                            }
                            if ui.add(egui::Button::new("❌").fill(Color32::TRANSPARENT)).clicked() {
                                app.new_roster_logo = None;
                                app.logo_textures.remove("__new_roster_logo");
                            }
                        });
                    } else {
                        if ui.add(egui::Button::new(RichText::new("🖼 Change Logo").color(theme::TEXT_PRIMARY()))
                            .fill(theme::BG_CARD())
                            .corner_radius(theme::button_rounding())
                        ).clicked() {
                            open_dialog = true;
                        }
                    }

                    if open_dialog {
                        app.image_picker_open = true;
                        app.image_picker_target = Some(crate::app::ImageTarget::NewRosterLogo);
                        app.image_picker_query = app.new_roster_name.clone();
                    }
                    
                    ui.add_space(16.0);
                    
                    let btn = egui::Button::new(RichText::new("+ Add Team").color(theme::BG_DARK()).strong())
                        .fill(theme::ACCENT_BRONZE())
                        .corner_radius(theme::button_rounding())
                        .min_size(Vec2::new(100.0, 32.0));
                        
                    if ui.add(btn).clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
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
                });
            }
            
            ui.add_space(12.0);
            
            egui::ScrollArea::vertical().id_salt("roster_scroll").show(ui, |ui| {
                let rosters = app.global_rosters.clone();
                for (idx, r) in rosters.into_iter().enumerate() {
                    let mut is_clicked = false;
                    
                    // Pre-load logo if needed
                    if let Some(ref logo_data) = r.logo_data {
                        if !app.logo_textures.contains_key(&r.id) {
                            if let Some(texture) = crate::app::TourviaApp::decode_logo(ui.ctx(), &r.id, logo_data) {
                                app.logo_textures.insert(r.id.clone(), texture);
                            }
                        }
                    }

                    ui.horizontal(|ui| {
                        let rect = ui.allocate_space(Vec2::new(ui.available_width(), 44.0)).1;
                        let response = ui.interact(rect, egui::Id::new(format!("roster_row_{}", r.id)), egui::Sense::click());
                        if response.hovered() {
                            ui.painter().rect_filled(rect, 4.0, theme::BG_CARD_HOVER());
                        }
                        if response.clicked() {
                            is_clicked = true;
                        }

                        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
                        
                        child_ui.allocate_ui(Vec2::new(300.0, 44.0), |ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                // Draw Logo
                                if let Some(texture) = app.logo_textures.get(&r.id) {
                                    ui.add(egui::Image::new(texture).fit_to_exact_size(Vec2::new(28.0, 28.0)).corner_radius(14.0));
                                } else {
                                    // Placeholder
                                    let (rect, _resp) = ui.allocate_exact_size(Vec2::new(28.0, 28.0), egui::Sense::hover());
                                    ui.painter().circle_filled(rect.center(), 14.0, theme::BG_ELEVATED());
                                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "?", egui::FontId::proportional(14.0), theme::TEXT_MUTED());
                                }
                                
                                ui.add_space(8.0);
                                ui.label(RichText::new(&r.name).color(theme::TEXT_PRIMARY()).size(16.0).strong());
                                ui.add_space(4.0);
                                ui.label(RichText::new(format!("({})", r.game)).color(theme::TEXT_MUTED()).size(13.0));
                            });
                        });
                        
                        child_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(RichText::new("🗑").color(theme::ERROR()))
                                .fill(Color32::TRANSPARENT)
                                .corner_radius(theme::button_rounding())
                            ).clicked() {
                                use crate::domain::repositories::RosterRepository;
                                if let Err(e) = app.db.delete_roster(&r.id) {
                                    app.notifications.error(format!("Delete failed: {}", e));
                                } else {
                                    app.notifications.info(format!("Deleted {}", r.name));
                                    app.load_rosters();
                                }
                            }
                            
                            ui.add_space(8.0);
                            
                            if ui.add(egui::Button::new(RichText::new("🔍 Detail").color(theme::TEXT_PRIMARY()))
                                .fill(theme::BG_ELEVATED())
                                .corner_radius(theme::button_rounding())
                            ).clicked() {
                                is_clicked = true;
                            }
                        });
                    });
                    ui.separator();
                    
                    if is_clicked {
                        app.open_roster(idx);
                    }
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

    ui.horizontal(|ui| {
        ui.label(theme::heading_text(&format!("🏆 Team Detail: {}", roster.name)));
        
        ui.add_space(8.0);
        if ui.add(egui::Button::new(RichText::new("🖼 Change Logo").color(theme::TEXT_PRIMARY()))
            .fill(theme::BG_ELEVATED())
            .corner_radius(theme::button_rounding())
        ).clicked() {
            app.image_picker_open = true;
            app.image_picker_target = Some(crate::app::ImageTarget::ExistingRosterLogo(roster.id.clone()));
            app.image_picker_query = roster.name.clone();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new(RichText::new("⬅ Back to Database").color(theme::TEXT_PRIMARY()))
                .fill(theme::BG_CARD_HOVER())
                .corner_radius(theme::button_rounding())
            ).clicked() {
                app.close_roster();
            }
        });
    });
    
    ui.add_space(20.0);

    // Add Member Section
    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.label(theme::subheading_text("Add Team Member"));
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("Player Name:").color(theme::TEXT_MUTED()));
                let name_edit = egui::TextEdit::singleline(&mut app.new_member_name).desired_width(200.0);
                let resp = ui.add(name_edit);
                
                ui.add_space(16.0);

                let mut open_member_dialog = false;

                if app.new_member_photo.is_some() {
                    if let Some(texture) = app.member_photo_textures.get("__new_member_photo") {
                        ui.add(egui::Image::new(texture).fit_to_exact_size(Vec2::new(32.0, 32.0)).corner_radius(8.0));
                    }
                    if ui.add(egui::Button::new(RichText::new("📷 Change").color(theme::TEXT_PRIMARY()))
                        .fill(theme::BG_CARD())
                        .corner_radius(theme::button_rounding())
                    ).clicked() {
                        open_member_dialog = true;
                    }
                    if ui.add(egui::Button::new("❌").fill(Color32::TRANSPARENT)).clicked() {
                        app.new_member_photo = None;
                        app.member_photo_textures.remove("__new_member_photo");
                    }
                } else {
                    if ui.add(egui::Button::new(RichText::new("📷 Select").color(theme::TEXT_PRIMARY()))
                        .fill(theme::BG_CARD())
                        .corner_radius(theme::button_rounding())
                    ).clicked() {
                        open_member_dialog = true;
                    }
                }

                if open_member_dialog {
                    app.image_picker_open = true;
                    app.image_picker_target = Some(crate::app::ImageTarget::NewMemberPhoto);
                    app.image_picker_query = app.new_member_name.clone();
                }
                
                ui.add_space(16.0);
                
                let btn = egui::Button::new(RichText::new("+ Add Member").color(theme::BG_DARK()).strong())
                    .fill(theme::ACCENT_BRONZE())
                    .corner_radius(theme::button_rounding())
                    .min_size(Vec2::new(100.0, 32.0));
                    
                if ui.add(btn).clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
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
        });

    ui.add_space(20.0);

    // Members List Section
    app.ensure_member_photos_loaded(ui.ctx());

    egui::Frame::new()
        .fill(theme::BG_PANEL())
        .stroke(theme::card_stroke())
        .corner_radius(theme::card_rounding())
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.label(theme::subheading_text("Team Members"));
            ui.add_space(12.0);

            if app.roster_members.is_empty() {
                ui.label(RichText::new("No members found. Add some above!").color(theme::TEXT_MUTED()));
            } else {
                egui::ScrollArea::vertical().id_salt("members_scroll").show(ui, |ui| {
                    let mut to_delete = None;

                    ui.horizontal_wrapped(|ui| {
                        for m in &app.roster_members {
                            let card_size = Vec2::new(120.0, 150.0);
                            ui.allocate_ui(card_size, |ui| {
                                let (rect, _) = ui.allocate_exact_size(card_size, egui::Sense::hover());
                                
                                // Draw background
                                ui.painter().rect_filled(rect, 8.0, theme::BG_ELEVATED());
                                ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(1.0, theme::BORDER_SUBTLE()), egui::StrokeKind::Inside);
                                
                                // Render photo (large)
                                let photo_rect = egui::Rect::from_min_size(
                                    rect.min + Vec2::new(10.0, 10.0),
                                    Vec2::new(100.0, 100.0),
                                );
                                
                                if m.profile_picture.is_some() {
                                    if let Some(tex) = app.member_photo_textures.get(&m.id) {
                                        let img = egui::Image::new(tex)
                                            .fit_to_exact_size(Vec2::new(100.0, 100.0))
                                            .corner_radius(6.0);
                                        ui.put(photo_rect, img);
                                    }
                                } else {
                                    ui.painter().rect_filled(photo_rect, 6.0, theme::BG_DARK());
                                    ui.painter().text(
                                        photo_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "👤",
                                        egui::FontId::proportional(32.0),
                                        theme::TEXT_MUTED(),
                                    );
                                }
                                
                                // Photo interaction
                                let photo_response = ui.interact(photo_rect, egui::Id::new(&m.id).with("photo"), egui::Sense::click());
                                if photo_response.hovered() {
                                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                                    ui.painter().rect_filled(photo_rect, 6.0, Color32::from_black_alpha(160));
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
                                
                                // Render name
                                let name_rect = egui::Rect::from_min_size(
                                    rect.min + Vec2::new(0.0, 115.0),
                                    Vec2::new(120.0, 35.0),
                                );
                                ui.painter().text(
                                    name_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    &m.name,
                                    egui::FontId::proportional(13.0),
                                    theme::TEXT_PRIMARY(),
                                );
                                
                                // Elegant delete button (top right inside the photo bounds)
                                let delete_rect = egui::Rect::from_min_size(
                                    photo_rect.right_top() + Vec2::new(-24.0, 4.0),
                                    Vec2::new(20.0, 20.0),
                                );
                                let mut delete_btn = egui::Button::new(RichText::new("✖").color(theme::TEXT_PRIMARY()).size(12.0))
                                    .fill(Color32::from_black_alpha(150))
                                    .corner_radius(10.0)
                                    .stroke(egui::Stroke::NONE)
                                    .min_size(Vec2::new(20.0, 20.0));
                                    
                                if ui.put(delete_rect, delete_btn).on_hover_text("Remove member").clicked() {
                                    to_delete = Some(m.id.clone());
                                }
                            });
                            ui.add_space(10.0);
                        }
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
