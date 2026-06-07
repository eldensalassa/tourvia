#![allow(unused_variables)]
use egui::{Align2, Color32, Pos2, Rect, Stroke, Ui, Vec2};
use egui::epaint::Shape;
use crate::app::{TourviaApp, BroadcastMode};
use crate::ui::theme;

// Control panel has been moved to match_panel.rs

pub fn render_overlay(app: &mut TourviaApp, ui: &mut Ui) {
    let screen_rect = ui.ctx().screen_rect();
    
    // Allow dragging the frameless window anywhere
    let response = ui.interact(screen_rect, ui.id().with("drag_area"), egui::Sense::drag());
    if response.dragged() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    
    // Hint text
    if app.broadcast_mode == BroadcastMode::Scoreboard {
        ui.painter().text(
            Pos2::new(10.0, 10.0),
            Align2::LEFT_TOP,
            "Drag to move",
            egui::FontId::proportional(12.0),
            Color32::from_black_alpha(80) // very subtle
        );
    }
    
    render_overlay_impl(app, ui, screen_rect, false);
}

fn render_overlay_impl(app: &mut TourviaApp, ui: &mut Ui, target_rect: Rect, is_preview: bool) {
    match app.broadcast_mode {
        BroadcastMode::Scoreboard => render_scoreboard_overlay(app, ui, target_rect, is_preview),
        BroadcastMode::Bracket => render_bracket_overlay(app, ui, target_rect, is_preview),
    }
}

fn render_scoreboard_overlay(app: &mut TourviaApp, ui: &mut Ui, target_rect: Rect, is_preview: bool) {
    let center_x = target_rect.center().x;
    
    // Scale parameters based on target width (assuming 800 base width for this specific viewport)
    let scale = target_rect.width() / 800.0;
    
    // Polished Esports Scoreboard Dimensions
    let total_width = 740.0 * scale;
    let main_bar_height = 60.0 * scale;
    let top_margin = 48.0 * scale;
    
    let center_y = target_rect.top() + top_margin + main_bar_height / 2.0;
    let y_top = target_rect.top() + top_margin;
    let y_bot = y_top + main_bar_height;
    
    let slant = 24.0 * scale; // Aggressive slant for esports look
    
    let bg_color = Color32::from_black_alpha(245); // Dark solid
    let score_bg_color = Color32::from_black_alpha(150); 
    let accent_color = theme::ACCENT_BRONZE();
    
    let left_edge = center_x - total_width / 2.0;
    let right_edge = center_x + total_width / 2.0;

    // Draw Main Bar as a slanted trapezoid (Gaming style)
    let main_points = vec![
        pos2(left_edge, y_bot),
        pos2(left_edge + slant, y_top),
        pos2(right_edge, y_top),
        pos2(right_edge - slant, y_bot),
    ];
    
    // Drop shadow
    let shadow_points = main_points.iter().map(|p| pos2(p.x, p.y + 6.0 * scale)).collect::<Vec<_>>();
    ui.painter().add(Shape::convex_polygon(shadow_points, Color32::from_black_alpha(120), Stroke::NONE));
    
    // Base shape
    ui.painter().add(Shape::convex_polygon(main_points, bg_color, Stroke::new(1.0, Color32::from_white_alpha(30))));
    
    // Bottom Accent Line (glow)
    let accent_points = vec![
        pos2(left_edge + 2.0, y_bot),
        pos2(left_edge + slant + 2.0, y_bot - 4.0 * scale),
        pos2(right_edge - slant - 2.0, y_bot - 4.0 * scale),
        pos2(right_edge - 2.0, y_bot),
    ];
    ui.painter().add(Shape::convex_polygon(accent_points, accent_color, Stroke::NONE));
    
    // Center Score Box Background (where scores go) - Parallelogram
    let score_box_w = 180.0 * scale;
    let sb_left = center_x - score_box_w / 2.0;
    let sb_right = center_x + score_box_w / 2.0;
    
    let score_box_points = vec![
        pos2(sb_left, y_bot),
        pos2(sb_left + slant, y_top),
        pos2(sb_right + slant, y_top), // Right slant follows the same angle
        pos2(sb_right, y_bot),
    ];
    ui.painter().add(Shape::convex_polygon(score_box_points, score_bg_color, Stroke::new(1.0, accent_color)));
    
    let active_match = app.matches.iter().find(|m| Some(&m.id) == app.selected_match.as_ref());
    
    if let Some(m) = active_match {
        let text_color = theme::TEXT_PRIMARY();
        let name_font = egui::FontId::new(26.0 * scale, egui::FontFamily::Name("Impact".into())); // Bold gaming font look
        let score_font = egui::FontId::new(42.0 * scale, egui::FontFamily::Name("Impact".into()));
        
        let p1_name = if m.player1_name.is_empty() { "TBD" } else { &m.player1_name };
        let p2_name = if m.player2_name.is_empty() { "TBD" } else { &m.player2_name };
        
        // Offset for slanted boxes
        let text_y = center_y - 2.0 * scale;
        
        let p1_name_str = p1_name.to_uppercase();
        let p2_name_str = p2_name.to_uppercase();
        
        // Calculate max available width for names to prevent overlapping with logos
        let logo_size = 44.0 * scale;
        
        // P1 Available Space
        let p1_logo_right = left_edge + slant + 24.0 * scale + logo_size / 2.0;
        let p1_name_right = sb_left - 24.0 * scale;
        let max_p1_width = (p1_name_right - p1_logo_right) - 12.0 * scale; // 12px padding
        
        let mut p1_font_size = 26.0 * scale;
        let mut p1_font = egui::FontId::new(p1_font_size, egui::FontFamily::Name("Impact".into()));
        while ui.painter().layout_no_wrap(p1_name_str.clone(), p1_font.clone(), text_color).rect.width() > max_p1_width && p1_font_size > 12.0 * scale {
            p1_font_size -= 1.0 * scale;
            p1_font = egui::FontId::new(p1_font_size, egui::FontFamily::Name("Impact".into()));
        }
        
        // P2 Available Space
        let p2_logo_left = right_edge - slant - 24.0 * scale - logo_size / 2.0;
        let p2_name_left = sb_right + slant + 24.0 * scale;
        let max_p2_width = (p2_logo_left - p2_name_left) - 12.0 * scale;
        
        let mut p2_font_size = 26.0 * scale;
        let mut p2_font = egui::FontId::new(p2_font_size, egui::FontFamily::Name("Impact".into()));
        while ui.painter().layout_no_wrap(p2_name_str.clone(), p2_font.clone(), text_color).rect.width() > max_p2_width && p2_font_size > 12.0 * scale {
            p2_font_size -= 1.0 * scale;
            p2_font = egui::FontId::new(p2_font_size, egui::FontFamily::Name("Impact".into()));
        }
        
        // P1 Name (Left side)
        ui.painter().text(
            Pos2::new(p1_name_right, text_y),
            Align2::RIGHT_CENTER,
            p1_name_str,
            p1_font,
            text_color
        );
        
        // P2 Name (Right side)
        ui.painter().text(
            Pos2::new(p2_name_left, text_y),
            Align2::LEFT_CENTER,
            p2_name_str,
            p2_font,
            text_color
        );
        
        // Scores
        let p1_score_pos = Pos2::new(sb_left + slant/2.0 + 35.0 * scale, text_y);
        let p2_score_pos = Pos2::new(sb_right + slant/2.0 - 35.0 * scale, text_y);
        
        ui.painter().text(
            p1_score_pos, Align2::CENTER_CENTER, m.score1.to_string(), score_font.clone(), theme::TEXT_PRIMARY()
        );
        ui.painter().text(
            p2_score_pos, Align2::CENTER_CENTER, m.score2.to_string(), score_font.clone(), theme::TEXT_PRIMARY()
        );
        
        // VS Text / Separator in the very middle of scores
        ui.painter().text(
            Pos2::new(center_x + slant/2.0, text_y),
            Align2::CENTER_CENTER,
            "VS",
            egui::FontId::new(20.0 * scale, egui::FontFamily::Name("Impact".into())),
            accent_color
        );
        
        // Match Timer Badge (Hanging below the center)
        let timer_width = 110.0 * scale;
        let timer_height = 32.0 * scale;
        
        let timer_points = vec![
            pos2(center_x - timer_width/2.0 + slant/2.0, y_bot),
            pos2(center_x + timer_width/2.0 + slant/2.0, y_bot),
            pos2(center_x + timer_width/2.0 - slant/2.0, y_bot + timer_height),
            pos2(center_x - timer_width/2.0 - slant/2.0, y_bot + timer_height),
        ];
        
        ui.painter().add(Shape::convex_polygon(timer_points, theme::BG_CARD(), Stroke::new(1.0, accent_color)));
        
        let mins = app.broadcast_timer_seconds / 60;
        let secs = app.broadcast_timer_seconds % 60;
        ui.painter().text(
            pos2(center_x, y_bot + timer_height/2.0),
            Align2::CENTER_CENTER,
            format!("{:02}:{:02}", mins, secs),
            egui::FontId::new(20.0 * scale, egui::FontFamily::Name("Impact".into())),
            if app.broadcast_timer_running { theme::SUCCESS() } else { theme::TEXT_PRIMARY() }
        );
        
        // Logos
        let logo_size = 44.0 * scale;
        if let Some(ref p1_id) = m.player1_id {
            if let Some(texture) = app.logo_textures.get(p1_id) {
                let logo_rect = Rect::from_center_size(
                    Pos2::new(left_edge + slant + 24.0 * scale, text_y),
                    Vec2::new(logo_size, logo_size)
                );
                ui.painter().image(texture.id(), logo_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            }
        }
        
        if let Some(ref p2_id) = m.player2_id {
            if let Some(texture) = app.logo_textures.get(p2_id) {
                let logo_rect = Rect::from_center_size(
                    Pos2::new(right_edge - slant - 24.0 * scale, text_y),
                    Vec2::new(logo_size, logo_size)
                );
                ui.painter().image(texture.id(), logo_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            }
        }
        
        // Tournament Logo (Floating top center)
        if let Some(ref t) = app.active_tournament {
            if let Some(texture) = app.tournament_logo_textures.get(&t.id) {
                let t_logo_size = 56.0 * scale;
                let logo_rect = Rect::from_center_size(
                    Pos2::new(center_x + slant/2.0, y_top - t_logo_size / 2.0 + 12.0 * scale),
                    Vec2::new(t_logo_size, t_logo_size)
                );
                
                // Hexagon background for logo to fit the gaming vibe
                let hex_r = t_logo_size / 2.0 + 8.0 * scale;
                let mut hex_points = Vec::new();
                for i in 0..6 {
                    let angle = std::f32::consts::PI / 3.0 * (i as f32) + std::f32::consts::PI / 6.0;
                    hex_points.push(pos2(logo_rect.center().x + hex_r * angle.cos(), logo_rect.center().y + hex_r * angle.sin()));
                }
                ui.painter().add(Shape::convex_polygon(hex_points, bg_color, Stroke::new(2.0 * scale, accent_color)));
                
                ui.painter().image(
                    texture.id(),
                    logo_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE
                );
            }
        }
        
    } else {
        // Fallback if no match selected
        ui.painter().text(
            pos2(center_x, target_rect.center().y),
            Align2::CENTER_CENTER,
            "NO ACTIVE MATCH",
            egui::FontId::new(24.0 * scale, egui::FontFamily::Name("Impact".into())),
            theme::TEXT_MUTED()
        );
    }
}

fn render_bracket_overlay(app: &mut TourviaApp, ui: &mut Ui, target_rect: Rect, is_preview: bool) {
    if !is_preview {
        ui.painter().rect_filled(target_rect, 0.0, Color32::from_black_alpha(200)); // Dark overlay over chroma
    }
    
    let scale = target_rect.width() / 1280.0;
    ui.painter().text(
        target_rect.center(),
        Align2::CENTER_CENTER,
        "Bracket View (Broadcast Mode) - Preview",
        egui::FontId::proportional(32.0 * scale),
        theme::TEXT_PRIMARY()
    );
}

fn pos2(x: f32, y: f32) -> Pos2 {
    Pos2::new(x, y)
}
