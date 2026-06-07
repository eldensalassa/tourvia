use egui::{self, Color32, RichText, Stroke, Ui, Vec2};

use crate::app::TourviaApp;
use crate::domain::tournament::TournamentType;
use crate::ui::theme;

/// Render the tournament creation form.
pub fn render(app: &mut TourviaApp, ui: &mut Ui) {
    ui.add_space(12.0);

    // Header with back button
    ui.horizontal(|ui| {
        if ui.add(
            egui::Button::new(RichText::new("< Back").color(theme::TEXT_SECONDARY()).size(14.0))
                .fill(Color32::TRANSPARENT),
        ).clicked() {
            app.go_to_dashboard();
        }
        ui.add_space(8.0);
        ui.label(theme::heading_text("Create New Tournament"));
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(20.0);

    let available_width = ui.available_width();
    let form_width = (available_width * 0.55).min(520.0);

    ui.vertical_centered(|ui| {
        ui.set_max_width(form_width);

        egui::Frame::new()
            .fill(theme::BG_CARD())
            .stroke(theme::card_stroke())
            .corner_radius(theme::card_rounding())
            .inner_margin(egui::Margin::same(28))
            .show(ui, |ui| {
                ui.set_min_width(form_width - 56.0);

                // ─── Tournament Name ────────────────
                ui.label(theme::section_header("TOURNAMENT NAME"));
                ui.add_space(4.0);
                let name_resp = egui::Frame::new()
                    .fill(theme::BG_DARK())
                    .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE()))
                    .corner_radius(4)
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        let name_edit = egui::TextEdit::singleline(&mut app.new_tournament_name)
                            .hint_text("e.g., Grand Championship 2026")
                            .desired_width(f32::INFINITY)
                            .font(egui::FontId::proportional(15.0))
                            .text_color(theme::TEXT_PRIMARY())
                            .background_color(theme::BG_DARK())
                            .frame(false)
                            .margin(egui::Margin::symmetric(0, 4));
                        ui.add_sized(Vec2::new(ui.available_width(), 30.0), name_edit)
                    })
                    .inner;
                if app.new_tournament_name.trim().is_empty() && name_resp.lost_focus() {
                    ui.label(RichText::new("Name is required").size(10.0).color(theme::ERROR()));
                }

                ui.add_space(18.0);

                // ─── Game Name ──────────────────────
                ui.label(theme::section_header("GAME / DISCIPLINE"));
                ui.add_space(4.0);
                
                if app.global_games.is_empty() {
                    ui.label(RichText::new("No games registered in Global Roster.").color(theme::WARNING()));
                } else {
                    if app.new_tournament_game.is_empty() {
                        app.new_tournament_game = app.global_games[0].name.clone();
                    }
                    let input_fill = theme::BG_DARK();
                    let field_width = ui.available_width();
                    ui.scope(|ui| {
                        let stroke = Stroke::new(1.0, theme::BORDER_SUBTLE());
                        let rounding = egui::CornerRadius::same(4);
                        ui.spacing_mut().menu_margin = egui::Margin::ZERO;
                        let visuals = ui.visuals_mut();
                        visuals.extreme_bg_color = input_fill;
                        visuals.window_fill = input_fill;
                        visuals.window_stroke = Stroke::NONE;
                        visuals.window_corner_radius = egui::CornerRadius::same(4);
                        visuals.menu_corner_radius = egui::CornerRadius::same(4);
                        visuals.popup_shadow = egui::Shadow::NONE;
                        visuals.panel_fill = input_fill;
                        visuals.widgets.inactive.bg_fill = input_fill;
                        visuals.widgets.inactive.weak_bg_fill = input_fill;
                        visuals.widgets.inactive.bg_stroke = stroke;
                        visuals.widgets.inactive.corner_radius = rounding;
                        visuals.widgets.hovered.bg_fill = input_fill;
                        visuals.widgets.hovered.weak_bg_fill = input_fill;
                        visuals.widgets.hovered.bg_stroke = stroke;
                        visuals.widgets.hovered.corner_radius = rounding;
                        visuals.widgets.active.bg_fill = input_fill;
                        visuals.widgets.active.weak_bg_fill = input_fill;
                        visuals.widgets.active.bg_stroke = stroke;
                        visuals.widgets.active.fg_stroke = Stroke::new(1.0, theme::TEXT_PRIMARY());
                        visuals.widgets.active.corner_radius = rounding;
                        visuals.widgets.open.bg_fill = input_fill;
                        visuals.widgets.open.weak_bg_fill = input_fill;
                        visuals.widgets.open.bg_stroke = stroke;
                        visuals.widgets.open.fg_stroke = Stroke::new(1.0, theme::TEXT_PRIMARY());
                        visuals.widgets.open.corner_radius = rounding;
                        visuals.selection.bg_fill = theme::ACCENT_BRONZE();
                        visuals.selection.stroke = Stroke::new(1.0, theme::ACCENT_BRONZE());

                        egui::ComboBox::from_id_salt("tournament_game_combo")
                            .selected_text(
                                RichText::new(&app.new_tournament_game)
                                    .color(theme::TEXT_PRIMARY()),
                            )
                            .width(field_width)
                            .height(148.0)
                            .truncate()
                            .show_ui(ui, |ui| {
                                for g in &app.global_games {
                                    let is_selected = app.new_tournament_game == g.name;
                                    let row_size = Vec2::new(ui.available_width(), 28.0);
                                    let (rect, response) =
                                        ui.allocate_exact_size(row_size, egui::Sense::click());

                                    if response.hovered() {
                                        ui.painter().rect_filled(
                                            rect,
                                            4.0,
                                            theme::ACCENT_BRONZE(),
                                        );
                                    }

                                    let text_color = if response.hovered() {
                                        theme::BG_DARK()
                                    } else if is_selected {
                                        theme::ACCENT_BRONZE_LIGHT()
                                    } else {
                                        theme::TEXT_PRIMARY()
                                    };

                                    ui.painter().text(
                                        rect.left_center() + Vec2::new(10.0, 0.0),
                                        egui::Align2::LEFT_CENTER,
                                        &g.name,
                                        egui::FontId::proportional(13.0),
                                        text_color,
                                    );

                                    if response.clicked() {
                                        app.new_tournament_game = g.name.clone();
                                    }
                                }
                            });
                        });
                }

                ui.add_space(18.0);

                // ─── Tournament Logo ────────────────
                ui.label(theme::section_header("TOURNAMENT LOGO (OPTIONAL)"));
                ui.add_space(4.0);
                
                ui.horizontal(|ui| {
                    if app.new_tournament_logo.is_some() {
                        ui.label(RichText::new("Image selected").color(theme::SUCCESS()));
                        if ui.add(egui::Button::new(RichText::new("Remove").color(theme::ERROR())).fill(Color32::TRANSPARENT)).clicked() {
                            app.new_tournament_logo = None;
                        }
                    } else {
                        if ui.add(egui::Button::new("Browse Image").fill(theme::BG_ELEVATED())).clicked() {
                            app.image_picker_open = true;
                            app.image_picker_target = Some(crate::app::ImageTarget::NewTournamentLogo);
                        }
                    }
                });

                ui.add_space(18.0);

                // ─── Description ────────────────────
                ui.label(theme::section_header("DESCRIPTION (OPTIONAL)"));
                ui.add_space(4.0);
                let desc_edit = egui::TextEdit::multiline(&mut app.new_tournament_description)
                    .hint_text("Tournament description, rules, prize pool, etc.")
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .font(egui::FontId::proportional(13.0))
                    .text_color(theme::TEXT_PRIMARY());
                ui.add(desc_edit);

                ui.add_space(18.0);

                // ─── Tournament Type ────────────────
                ui.label(theme::section_header("TOURNAMENT FORMAT"));
                ui.add_space(6.0);

                ui.horizontal_wrapped(|ui| {
                    type_button(ui, app, TournamentType::SingleElimination, "Single Elimination", theme::ACCENT_BRONZE());
                    type_button(ui, app, TournamentType::DoubleElimination, "Double Elimination", theme::ACCENT_BRONZE_LIGHT());
                    type_button(ui, app, TournamentType::RoundRobin, "Round Robin", theme::ACCENT_BRONZE_DARK());
                });

                ui.add_space(6.0);
                // Type description
                let type_desc = match app.new_tournament_type {
                    TournamentType::SingleElimination => "Lose once and you're out. Fast & decisive.",
                    TournamentType::DoubleElimination => "Two chances — lose twice before elimination.",
                    TournamentType::RoundRobin => "Everyone plays everyone. Best overall record wins.",
                };
                ui.label(RichText::new(type_desc).size(11.0).color(theme::TEXT_MUTED()).italics());

                ui.add_space(28.0);

                // ─── Create Button ──────────────────
                let can_create = !app.new_tournament_name.trim().is_empty();
                let create_btn = egui::Button::new(
                    RichText::new("Create Tournament")
                        .size(15.0)
                        .color(if can_create { theme::BG_DARK() } else { theme::TEXT_MUTED() })
                        .strong(),
                )
                .fill(if can_create { theme::ACCENT_BRONZE() } else { theme::BG_CARD_HOVER() })
                .corner_radius(theme::button_rounding());

                ui.add_enabled_ui(can_create, |ui| {
                    if ui.add_sized(Vec2::new(ui.available_width(), 42.0), create_btn).clicked() {
                        app.create_tournament();
                    }
                });


            });
    });
}

fn type_button(ui: &mut Ui, app: &mut TourviaApp, t_type: TournamentType, label: &str, color: Color32) {
    let is_selected = app.new_tournament_type == t_type;
    let btn = egui::Button::new(
        RichText::new(label)
            .size(12.0)
            .color(if is_selected { theme::BG_DARK() } else { theme::TEXT_SECONDARY() })
            .strong(),
    )
    .fill(if is_selected { color } else { theme::BG_CARD_HOVER() })
    .corner_radius(theme::button_rounding())
    .min_size(Vec2::new(0.0, 32.0));

    if ui.add(btn).clicked() {
        app.new_tournament_type = t_type;
    }
}
