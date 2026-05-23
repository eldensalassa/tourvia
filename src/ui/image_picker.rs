use egui::{Color32, RichText, Vec2, Stroke};
use crate::app::{TourviaApp, ImageTarget};
use crate::ui::theme;

pub fn render_modal(app: &mut TourviaApp, ctx: &egui::Context) {
    if !app.image_picker_open {
        return;
    }

    if let Some(rx) = &app.thumbnail_fetch_rx {
        while let Ok((url, color_image)) = rx.try_recv() {
            let texture = ctx.load_texture(&url, color_image, egui::TextureOptions::LINEAR);
            app.image_picker_thumbnails.insert(url, texture);
        }
    }

    // ── Process async channels ───────────────────────────
    if let Some(rx) = &app.image_fetch_rx {
        if let Ok(result) = rx.try_recv() {
            app.image_fetch_rx = None;
            app.image_picker_loading = false;
            match result {
                Ok(images) => {
                    app.image_picker_results = Some(images.clone());

                    // Clear previous thumbnails
                    app.image_picker_thumbnails.clear();
                    
                    // Spawn background thread to fetch thumbnails
                    let (tx, rx) = std::sync::mpsc::channel();
                    app.thumbnail_fetch_rx = Some(rx);
                    
                    for item in images {
                        let url = item.thumbnail.clone();
                        let tx = tx.clone();
                        let ctx_clone = ctx.clone();
                        std::thread::spawn(move || {
                            if let Ok(bytes) = crate::utils::scraper::download_image(&url) {
                                if let Ok(img) = image::load_from_memory(&bytes) {
                                    let size = [img.width() as _, img.height() as _];
                                    let image_buffer = img.to_rgba8();
                                    let pixels = image_buffer.as_flat_samples();
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        pixels.as_slice(),
                                    );
                                    let _ = tx.send((url, color_image));
                                    ctx_clone.request_repaint();
                                }
                            }
                        });
                    }
                }
                Err(e) => {
                    app.notifications.error(format!("Search failed: {}", e));
                }
            }
        }
    }

    if let Some(rx) = &app.image_download_rx {
        if let Ok(result) = rx.try_recv() {
            app.image_download_rx = None;
            app.image_picker_loading = false;
            app.image_picker_open = false;

            match result {
                Ok(data) => {
                    if let Ok(processed) = crate::app::TourviaApp::process_logo(&data) {
                        apply_image_to_target(app, ctx, processed);
                    } else {
                        app.notifications.error("Failed to process downloaded image.");
                    }
                }
                Err(e) => {
                    app.notifications.error(format!("Download failed: {}", e));
                }
            }
        }
    }

    if !app.image_picker_open {
        return;
    }

    // ── Modal Window ─────────────────────────────────────
    egui::Window::new("ImagePickerModal")
        .title_bar(false)
        .collapsible(false)
        .resizable(true)
        .default_width(720.0)
        .default_height(560.0)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(
            egui::Frame::new()
                .fill(theme::BG_PANEL())
                .stroke(theme::card_stroke())
                .corner_radius(theme::card_rounding())
                .inner_margin(0.0)
        )
        .show(ctx, |ui| {
            // ── Custom Title Bar ─────────────────────────
            egui::Frame::new()
                .fill(theme::BG_PANEL())
                .inner_margin(egui::Margin { left: 24, right: 16, top: 16, bottom: 12 })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("🖼 Select Image")
                                .font(egui::FontId::proportional(20.0))
                                .color(theme::TEXT_PRIMARY())
                                .strong()
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(
                                egui::Button::new(RichText::new("✖").size(16.0).color(theme::TEXT_MUTED()))
                                    .fill(Color32::TRANSPARENT)
                            ).clicked() {
                                app.image_picker_open = false;
                            }
                        });
                    });
                });

            // ── Search Bar ───────────────────────────────
            egui::Frame::new()
                .fill(theme::BG_PANEL())
                .inner_margin(egui::Margin { left: 24, right: 24, top: 0, bottom: 12 })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔍").size(14.0));

                        let search_box = ui.add_sized(
                            [ui.available_width() - 100.0, 32.0],
                            egui::TextEdit::singleline(&mut app.image_picker_query)
                                .hint_text("Search images (e.g. RRQ logo png)")
                                .font(egui::FontId::proportional(13.0))
                                .text_color(theme::TEXT_PRIMARY())
                        );

                        let search_btn = egui::Button::new(
                            RichText::new("Search").color(theme::BG_DARK()).strong().size(12.0)
                        )
                            .fill(theme::ACCENT_BRONZE())
                            .corner_radius(theme::button_rounding())
                            .min_size(Vec2::new(80.0, 32.0));

                        let search_clicked = ui.add(search_btn).clicked();

                        if search_clicked || (search_box.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            trigger_search(app);
                        }
                    });
                });

            ui.add(egui::Separator::default().spacing(0.0));

            // ── Content Area ─────────────────────────────
            egui::Frame::new()
                .fill(theme::BG_DARK())
                .inner_margin(16.0)
                .show(ui, |ui| {
                    let content_height = 360.0;
                    ui.set_min_height(content_height);

                    if app.image_picker_loading {
                        // ── Loading State ────────────────
                        ui.vertical_centered(|ui| {
                            ui.add_space(content_height / 2.0 - 40.0);
                            ui.spinner();
                            ui.add_space(12.0);
                            ui.label(theme::label_text("Loading..."));
                        });
                    } else if let Some(results) = &app.image_picker_results {
                        if results.is_empty() {
                            // ── Empty State ──────────────
                            ui.vertical_centered(|ui| {
                                ui.add_space(content_height / 2.0 - 40.0);
                                ui.label(RichText::new("🔍").size(40.0));
                                ui.add_space(8.0);
                                ui.label(theme::label_text("No images found. Try a different query."));
                            });
                        } else {
                            // ── Image Grid ───────────────
                            egui::ScrollArea::vertical()
                                .id_salt("image_picker_scroll")
                                .auto_shrink([false; 2])
                                .max_height(content_height)
                                .show(ui, |ui| {
                                    render_image_grid(app, ui);
                                });
                        }
                    } else {
                        // ── Initial State ────────────────
                        ui.vertical_centered(|ui| {
                            ui.add_space(content_height / 2.0 - 50.0);
                            ui.label(RichText::new("🖼").size(48.0));
                            ui.add_space(12.0);
                            ui.label(theme::label_text("Search for images online"));
                            ui.add_space(4.0);
                            ui.label(theme::small_text("or browse a local file from your computer"));
                        });
                    }
                });

            ui.add(egui::Separator::default().spacing(0.0));

            // ── Bottom Action Bar ────────────────────────
            egui::Frame::new()
                .fill(theme::BG_PANEL())
                .inner_margin(egui::Margin { left: 24, right: 24, top: 12, bottom: 16 })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Left side: Browse Local
                        if ui.add(
                            egui::Button::new(
                                RichText::new("📁 Browse Local File").color(theme::TEXT_PRIMARY()).size(12.0)
                            )
                                .fill(theme::BG_CARD_HOVER())
                                .corner_radius(theme::button_rounding())
                                .min_size(Vec2::new(140.0, 34.0))
                        ).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                                .pick_file()
                            {
                                if let Ok(data) = std::fs::read(&path) {
                                    if let Ok(processed) = crate::app::TourviaApp::process_logo(&data) {
                                        apply_image_to_target(app, ctx, processed);
                                        app.image_picker_open = false;
                                    }
                                }
                            }
                        }

                        // Right side: Select + Cancel
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Cancel
                            if ui.add(
                                egui::Button::new(
                                    RichText::new("Cancel").color(theme::TEXT_SECONDARY()).size(12.0)
                                )
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::new(1.0, theme::BORDER()))
                                    .corner_radius(theme::button_rounding())
                                    .min_size(Vec2::new(80.0, 34.0))
                            ).clicked() {
                                app.image_picker_open = false;
                            }

                            ui.add_space(8.0);

                            // Select
                            let selected_url: String = ui.memory_mut(|mem| {
                                mem.data.get_temp(egui::Id::new("selected_image_url")).unwrap_or_default()
                            });

                            let has_selection = !selected_url.is_empty();

                            let select_btn = egui::Button::new(
                                RichText::new("✓ Select")
                                    .color(if has_selection { theme::BG_DARK() } else { theme::TEXT_MUTED() })
                                    .strong()
                                    .size(12.0)
                            )
                                .fill(if has_selection { theme::ACCENT_BRONZE() } else { theme::BG_ELEVATED() })
                                .corner_radius(theme::button_rounding())
                                .min_size(Vec2::new(90.0, 34.0));

                            if ui.add(select_btn).clicked() {
                                if has_selection {
                                    trigger_download(app, &selected_url);
                                } else {
                                    app.notifications.info("Click on an image to select it first.");
                                }
                            }
                        });
                    });
                });
        });
}

// ── Helpers ──────────────────────────────────────────────

fn trigger_search(app: &mut TourviaApp) {
    if app.image_picker_query.trim().is_empty() {
        return;
    }
    app.image_picker_loading = true;
    app.image_picker_results = None;

    let mut query = app.image_picker_query.clone();
    
    // Append game context for better accuracy
    let game_context = match &app.image_picker_target {
        Some(crate::app::ImageTarget::NewRosterLogo) => Some(app.new_roster_game.clone()),
        Some(crate::app::ImageTarget::ExistingRosterLogo(id)) => {
            app.global_rosters.iter().find(|r| &r.id == id).map(|r| r.game.clone())
        }
        _ => None,
    };
    
    if let Some(game) = game_context {
        if !game.is_empty() && !query.to_lowercase().contains(&game.to_lowercase()) {
            query.push_str(" ");
            query.push_str(&game);
        }
    }
    
    // Explicitly append liquipedia to force Bing to search there
    if !query.to_lowercase().contains("liquipedia") && !query.to_lowercase().contains("wikipedia") {
        query.push_str(" liquipedia");
    }

    let (tx, rx) = std::sync::mpsc::channel();
    app.image_fetch_rx = Some(rx);

    std::thread::spawn(move || {
        let res = crate::utils::scraper::fetch_images_bing(&query);
        let _ = tx.send(res);
    });
}

fn trigger_download(app: &mut TourviaApp, url: &str) {
    app.image_picker_loading = true;
    let (tx, rx) = std::sync::mpsc::channel();
    app.image_download_rx = Some(rx);
    let url = url.to_string();

    std::thread::spawn(move || {
        let res = crate::utils::scraper::download_image(&url);
        let _ = tx.send(res);
    });
}

fn render_image_grid(app: &mut TourviaApp, ui: &mut egui::Ui) {
    let results = match &app.image_picker_results {
        Some(r) => r.clone(),
        None => return,
    };

    let mut clicked_url: Option<String> = None;
    let mut double_clicked_url: Option<String> = None;

    let selected_url: String = ui.memory_mut(|mem| {
        mem.data.get_temp(egui::Id::new("selected_image_url")).unwrap_or_default()
    });

    // Result count header
    ui.horizontal(|ui| {
        ui.label(theme::small_text(&format!("{} image(s) found", results.len())));
        if !selected_url.is_empty() {
            ui.label(
                RichText::new(" • 1 selected")
                    .size(11.0)
                    .color(theme::ACCENT_BRONZE())
            );
        }
    });
    ui.add_space(8.0);

    let item_size = 120.0;
    let card_height = item_size;
    let spacing = 10.0;
    let columns = ((ui.available_width() + spacing) / (item_size + spacing)).floor() as usize;
    let columns = columns.max(1);

    for row in results.chunks(columns) {
        ui.horizontal(|ui| {
            for item in row {
                let is_selected = selected_url == item.url;

                ui.allocate_ui(Vec2::new(item_size, card_height), |ui| {
                    // Card container
                    let card_rect = egui::Rect::from_min_size(
                        ui.cursor().left_top(),
                        Vec2::new(item_size, card_height),
                    );

                    let response = ui.interact(
                        card_rect,
                        egui::Id::new(&item.url),
                        egui::Sense::click(),
                    );

                    // Card background with state-based styling
                    let (bg, border_stroke) = if is_selected {
                        (
                            theme::ACCENT_BRONZE().linear_multiply(0.15),
                            Stroke::new(2.0, theme::ACCENT_BRONZE()),
                        )
                    } else if response.hovered() {
                        (
                            theme::BG_CARD_HOVER(),
                            Stroke::new(1.0, theme::BORDER()),
                        )
                    } else {
                        (
                            theme::BG_CARD(),
                            Stroke::new(1.0, theme::BORDER_SUBTLE()),
                        )
                    };

                    ui.painter().rect_filled(card_rect, theme::card_rounding(), bg);
                    ui.painter().rect_stroke(card_rect, theme::card_rounding(), border_stroke, egui::StrokeKind::Inside);

                    // Selected checkmark badge
                    if is_selected {
                        let badge_center = card_rect.right_top() + egui::vec2(-10.0, 10.0);
                        ui.painter().circle_filled(badge_center, 10.0, theme::ACCENT_BRONZE());
                        ui.painter().text(
                            badge_center,
                            egui::Align2::CENTER_CENTER,
                            "✓",
                            egui::FontId::proportional(12.0),
                            theme::BG_DARK(),
                        );
                    }

                    // Thumbnail area
                    let thumb_rect = egui::Rect::from_min_size(
                        card_rect.min + egui::vec2(8.0, 8.0),
                        Vec2::new(item_size - 16.0, item_size - 16.0),
                    );
                    
                    if let Some(texture) = app.image_picker_thumbnails.get(&item.thumbnail) {
                        ui.painter().image(
                            texture.id(),
                            thumb_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE
                        );
                    } else {
                        ui.painter().rect_filled(thumb_rect, 6.0, theme::BG_DARK().linear_multiply(0.5));
                        ui.painter().text(
                            thumb_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "🖼",
                            egui::FontId::proportional(24.0),
                            theme::TEXT_MUTED().linear_multiply(0.6),
                        );
                    }



                    // Advance the cursor past this card
                    ui.allocate_space(Vec2::new(item_size, card_height));

                    // Interaction
                    if response.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }
                    if response.clicked() {
                        clicked_url = Some(item.url.clone());
                    }
                    if response.double_clicked() {
                        double_clicked_url = Some(item.url.clone());
                    }
                });
            }
        });
        ui.add_space(spacing);
    }

    // Apply click actions
    if let Some(url) = clicked_url {
        ui.memory_mut(|mem| mem.data.insert_temp(egui::Id::new("selected_image_url"), url));
    }
    if let Some(url) = double_clicked_url {
        ui.memory_mut(|mem| mem.data.insert_temp(egui::Id::new("selected_image_url"), url.clone()));
        trigger_download(app, &url);
    }
}

// ── Target Application ──────────────────────────────────

fn apply_image_to_target(app: &mut TourviaApp, ctx: &egui::Context, processed: Vec<u8>) {
    let target = if let Some(t) = &app.image_picker_target {
        t.clone()
    } else {
        return;
    };

    use crate::domain::repositories::RosterRepository;

    match target {
        ImageTarget::NewRosterLogo => {
            app.new_roster_logo = Some(processed.clone());
            if let Some(texture) = crate::app::TourviaApp::decode_logo(ctx, "new_roster_logo", &processed) {
                app.logo_textures.insert("__new_roster_logo".to_string(), texture);
            }
        }
        ImageTarget::ExistingRosterLogo(roster_id) => {
            if let Some(r) = app.global_rosters.iter_mut().find(|r| r.id == roster_id) {
                r.logo_data = Some(processed.clone());
                if let Err(e) = app.db.update_roster(r) {
                    app.notifications.error(format!("Failed to update logo: {}", e));
                } else {
                    app.notifications.success("Logo updated!");
                    if let Some(texture) = crate::app::TourviaApp::decode_logo(ctx, &roster_id, &processed) {
                        app.logo_textures.insert(roster_id, texture);
                    }
                    if let Some(active) = &mut app.active_roster {
                        if active.id == r.id {
                            active.logo_data = r.logo_data.clone();
                        }
                    }
                }
            }
        }
        ImageTarget::NewMemberPhoto => {
            app.new_member_photo = Some(processed.clone());
            if let Some(texture) = crate::app::TourviaApp::decode_logo(ctx, "new_member_photo", &processed) {
                app.member_photo_textures.insert("__new_member_photo".to_string(), texture);
            }
        }
        ImageTarget::ExistingMemberPhoto(member_id) => {
            if let Some(m) = app.roster_members.iter_mut().find(|m| m.id == member_id) {
                m.profile_picture = Some(processed.clone());
                if let Err(e) = app.db.update_roster_member(m) {
                    app.notifications.error(format!("Failed to update photo: {}", e));
                } else {
                    app.notifications.success("Photo updated!");
                    if let Some(texture) = crate::app::TourviaApp::decode_logo(ctx, &member_id, &processed) {
                        app.member_photo_textures.insert(member_id.clone(), texture);
                    }
                }
            }
        }
    }
}
