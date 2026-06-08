use egui::{self, Align2, Color32, FontFamily, FontId, Pos2, Rect, RichText, Stroke, Vec2};
use egui::epaint::Shape;

use crate::app::TourviaApp;
use crate::domain::match_model::{Match, MatchStatus};
use crate::ui::theme;

pub fn render_display_window(app: &mut TourviaApp, ctx: &egui::Context) {
    ctx.request_repaint_after(std::time::Duration::from_secs(1));
    ensure_scoreboard_selection(app);

    let current_match = current_match(app);
    let builder = egui::ViewportBuilder::default()
        .with_title("Tourvia Scoreboard Display")
        .with_inner_size([1280.0, 720.0])
        .with_min_inner_size([900.0, 520.0])
        .with_resizable(true);

    let mut close_requested = false;
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("tourvia_scoreboard_display"),
        builder,
        |viewport_ctx, _class| {
            viewport_ctx.request_repaint_after(std::time::Duration::from_secs(1));
            if viewport_ctx.input(|i| i.viewport().close_requested()) {
                close_requested = true;
            }

            // Darker background for gaming aesthetic
            let bg_color = Color32::from_rgb(12, 16, 20);

            egui::CentralPanel::default()
                .frame(
                    egui::Frame::new()
                        .fill(bg_color)
                        .inner_margin(egui::Margin::symmetric(0, 0)),
                )
                .show(viewport_ctx, |ui| {
                    if let Some(m) = &current_match {
                        render_scoreboard(app, ui, m);
                    } else {
                        render_empty(ui);
                    }
                });
        },
    );

    if close_requested {
        app.scoreboard_display_window_open = false;
    }
}

fn render_scoreboard(app: &TourviaApp, ui: &mut egui::Ui, m: &Match) {
    let round_name = round_name(app, &m.round_id);
    let (status_text, status_color) = status_info(&m.status);

    let screen_rect = ui.available_rect_before_wrap();
    let scale = screen_rect.width() / 1280.0;
    
    // Background Decorations (Gaming Grid & Borders)
    let grid_color = Color32::from_white_alpha(3);
    let grid_size = 40.0 * scale;
    let mut x = screen_rect.left();
    while x < screen_rect.right() {
        ui.painter().line_segment([Pos2::new(x, screen_rect.top()), Pos2::new(x, screen_rect.bottom())], Stroke::new(1.0, grid_color));
        x += grid_size;
    }
    let mut y = screen_rect.top();
    while y < screen_rect.bottom() {
        ui.painter().line_segment([Pos2::new(screen_rect.left(), y), Pos2::new(screen_rect.right(), y)], Stroke::new(1.0, grid_color));
        y += grid_size;
    }
    
    // Top & Bottom Accent Bars
    ui.painter().rect_filled(
        Rect::from_min_max(screen_rect.left_top(), Pos2::new(screen_rect.right(), screen_rect.top() + 4.0 * scale)),
        0.0,
        theme::ACCENT_BRONZE()
    );
    ui.painter().rect_filled(
        Rect::from_min_max(Pos2::new(screen_rect.left(), screen_rect.bottom() - 4.0 * scale), screen_rect.right_bottom()),
        0.0,
        theme::ACCENT_BRONZE().linear_multiply(0.5)
    );
    
    // Top bar (Tournament & Round)
    ui.vertical_centered(|ui| {
        ui.add_space(16.0 * scale);
        
        if let Some(ref t) = app.active_tournament {
            if let Some(texture) = app.tournament_logo_textures.get(&t.id) {
                let logo_size = 48.0 * scale;
                ui.add(egui::Image::new(texture).fit_to_exact_size(Vec2::new(logo_size, logo_size)));
                ui.add_space(4.0 * scale);
            }
        }

        let t_name = app.active_tournament.as_ref().map(|t| t.name.clone()).unwrap_or_else(|| "Tourvia".to_string());
        
        ui.label(
            RichText::new(t_name.to_uppercase())
                .font(FontId::new(32.0 * scale, FontFamily::Name("Impact".into())))
                .color(theme::TEXT_PRIMARY())
        );
        ui.label(
            RichText::new(round_name.to_uppercase())
                .font(FontId::new(22.0 * scale, FontFamily::Name("Impact".into())))
                .color(theme::ACCENT_BRONZE())
        );
        ui.add_space(6.0 * scale);
        ui.label(
            RichText::new(status_text.to_uppercase())
                .font(FontId::new(18.0 * scale, FontFamily::Name("Impact".into())))
                .color(status_color)
        );
    });

    ui.add_space(24.0 * scale);

    let available_height = (ui.available_height() - 100.0 * scale).max(350.0 * scale);
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), available_height),
        egui::Sense::hover(),
    );
    
    let center_x = rect.center().x;
    
    // Background VS Watermark
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        "VS",
        FontId::new(380.0 * scale, FontFamily::Name("Impact".into())),
        Color32::from_white_alpha(5)
    );
    
    let gap = 50.0 * scale;
    let panel_width = 460.0 * scale;
    
    let p1_rect = Rect::from_min_max(
        Pos2::new(center_x - gap - panel_width, rect.top()),
        Pos2::new(center_x - gap, rect.bottom())
    );
    let p2_rect = Rect::from_min_max(
        Pos2::new(center_x + gap, rect.top()),
        Pos2::new(center_x + gap + panel_width, rect.bottom())
    );
    
    render_player_card(app, ui, p1_rect, m, 1, scale);
    render_player_card(app, ui, p2_rect, m, 2, scale);
    
    render_center_score_custom(app, ui, rect, m, scale);
    
    // Bottom "Next Up" section
    ui.add_space(30.0 * scale);
    render_next_match(app, ui, m, scale);
}

fn render_player_card(app: &TourviaApp, ui: &mut egui::Ui, rect: Rect, m: &Match, slot: i32, scale: f32) {
    let painter = ui.painter();
    
    let (id, name, score, is_winner) = if slot == 1 {
        (
            m.player1_id.as_deref(),
            player_name(&m.player1_name),
            m.score1,
            m.winner_id.is_some() && m.player1_id == m.winner_id,
        )
    } else {
        (
            m.player2_id.as_deref(),
            player_name(&m.player2_name),
            m.score2,
            m.winner_id.is_some() && m.player2_id == m.winner_id,
        )
    };
    
    let slant = 50.0 * scale;
    let bg = if is_winner {
        theme::SUCCESS().linear_multiply(0.12)
    } else {
        Color32::from_rgb(22, 26, 30)
    };
    
    let stroke_color = if is_winner { theme::SUCCESS() } else { theme::ACCENT_BRONZE().linear_multiply(0.4) };
    
    // Create the slanted polygon (V-shape gap in the middle)
    let points = if slot == 1 {
        vec![
            Pos2::new(rect.left() + slant, rect.top()),
            Pos2::new(rect.right(), rect.top()),
            Pos2::new(rect.right() - slant, rect.bottom()),
            Pos2::new(rect.left(), rect.bottom()),
        ]
    } else {
        vec![
            Pos2::new(rect.left(), rect.top()),
            Pos2::new(rect.right() - slant, rect.top()),
            Pos2::new(rect.right(), rect.bottom()),
            Pos2::new(rect.left() + slant, rect.bottom()),
        ]
    };
    
    // Shadow
    let shadow_points: Vec<_> = points.iter().map(|p| Pos2::new(p.x, p.y + 12.0 * scale)).collect();
    painter.add(Shape::convex_polygon(shadow_points, Color32::from_black_alpha(150), Stroke::NONE));
    
    // Card Base
    painter.add(Shape::convex_polygon(points, bg, Stroke::new(2.0 * scale, stroke_color)));
    
    // Tech Accents (small squares on the inner slanted edge)
    let accent_color = theme::BORDER_SUBTLE().linear_multiply(0.8);
    if slot == 1 {
        for i in 0..4 {
            let y = rect.top() + 30.0 * scale + (i as f32) * 16.0 * scale;
            let x = rect.right() - (y - rect.top()) / rect.height() * slant - 12.0 * scale;
            painter.rect_filled(Rect::from_center_size(Pos2::new(x, y), Vec2::new(6.0 * scale, 6.0 * scale)), 1.0 * scale, accent_color);
        }
    } else {
        for i in 0..4 {
            let y = rect.top() + 30.0 * scale + (i as f32) * 16.0 * scale;
            let x = rect.left() + (y - rect.top()) / rect.height() * slant + 12.0 * scale;
            painter.rect_filled(Rect::from_center_size(Pos2::new(x, y), Vec2::new(6.0 * scale, 6.0 * scale)), 1.0 * scale, accent_color);
        }
    }
    
    // Winner accent line and text
    if is_winner {
        let line_pts = if slot == 1 {
            [Pos2::new(rect.left() + slant/2.0, rect.bottom()), Pos2::new(rect.right() - slant, rect.bottom())]
        } else {
            [Pos2::new(rect.left() + slant, rect.bottom()), Pos2::new(rect.right() - slant/2.0, rect.bottom())]
        };
        painter.line_segment(line_pts, Stroke::new(6.0 * scale, theme::SUCCESS()));
        
        painter.text(
            Pos2::new(rect.center().x, rect.bottom() - 25.0 * scale),
            Align2::CENTER_CENTER,
            "WINNER",
            FontId::new(22.0 * scale, FontFamily::Name("Impact".into())),
            theme::SUCCESS()
        );
    }
    
    // Logo
    let logo_size = 130.0 * scale;
    let logo_center = Pos2::new(rect.center().x, rect.top() + logo_size/2.0 + 30.0 * scale);
    
    if let Some(id) = id {
        if let Some(texture) = app.logo_textures.get(id) {
            let logo_rect = Rect::from_center_size(logo_center, Vec2::new(logo_size, logo_size));
            painter.image(texture.id(), logo_rect, Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)), Color32::WHITE);
        } else {
            draw_logo_placeholder(painter, logo_center, logo_size, scale);
        }
    } else {
        draw_logo_placeholder(painter, logo_center, logo_size, scale);
    }
    
    // Name
    // Calculate adaptive font size for long names
    let max_width = 320.0 * scale;
    let mut font_size = 36.0 * scale;
    let mut font = FontId::new(font_size, FontFamily::Name("Impact".into()));
    while ui.painter().layout_no_wrap(name.to_uppercase(), font.clone(), Color32::WHITE).rect.width() > max_width && font_size > 18.0 * scale {
        font_size -= 2.0 * scale;
        font = FontId::new(font_size, FontFamily::Name("Impact".into()));
    }
    
    painter.text(
        Pos2::new(rect.center().x, rect.top() + logo_size + 65.0 * scale),
        Align2::CENTER_CENTER,
        name.to_uppercase(),
        font,
        if is_winner { theme::SUCCESS() } else { theme::TEXT_PRIMARY() }
    );
    
    // Score
    if m.status == MatchStatus::Completed || m.status == MatchStatus::InProgress {
        painter.text(
            Pos2::new(rect.center().x, rect.bottom() - 75.0 * scale),
            Align2::CENTER_CENTER,
            score.to_string(),
            FontId::new(100.0 * scale, FontFamily::Name("Impact".into())),
            if is_winner { theme::SUCCESS() } else { theme::TEXT_PRIMARY() }
        );
    }
}

fn render_center_score_custom(app: &TourviaApp, ui: &mut egui::Ui, rect: Rect, m: &Match, scale: f32) {
    let painter = ui.painter();
    let center = rect.center();
    
    // Draw hexagon in the center
    let hex_radius = 115.0 * scale; // Increased radius for more breathing room
    let mut hex_points = Vec::new();
    for i in 0..6 {
        let angle = std::f32::consts::PI / 3.0 * (i as f32) + std::f32::consts::PI / 6.0;
        hex_points.push(Pos2::new(center.x + hex_radius * angle.cos(), center.y + hex_radius * angle.sin()));
    }
    
    let hex_shadow: Vec<_> = hex_points.iter().map(|p| Pos2::new(p.x, p.y + 12.0 * scale)).collect();
    painter.add(Shape::convex_polygon(hex_shadow, Color32::from_black_alpha(150), Stroke::NONE));
    
    // Double border effect for gaming aesthetic
    let bg_color = Color32::from_rgb(30, 34, 38);
    painter.add(Shape::convex_polygon(hex_points.clone(), bg_color, Stroke::new(4.0 * scale, theme::ACCENT_BRONZE())));
    
    let mut inner_hex = Vec::new();
    for p in &hex_points {
        let dir = (*p - center).normalized();
        inner_hex.push(center + dir * (hex_radius - 8.0 * scale));
    }
    painter.add(Shape::convex_polygon(inner_hex, Color32::TRANSPARENT, Stroke::new(1.0 * scale, theme::ACCENT_BRONZE().linear_multiply(0.3))));
    
    // Status text offset
    let y_offset = if app.broadcast_timer_running || app.broadcast_timer_seconds > 0 { -24.0 } else { -10.0 };
    
    if m.status == MatchStatus::Completed || m.status == MatchStatus::InProgress {
        painter.text(
            Pos2::new(center.x, center.y + y_offset * scale),
            Align2::CENTER_CENTER,
            format!("{} - {}", m.score1, m.score2),
            FontId::new(60.0 * scale, FontFamily::Name("Impact".into())),
            theme::ACCENT_BRONZE()
        );
        let label = if m.status == MatchStatus::Completed { "FINAL" } else { "LIVE" };
        painter.text(
            Pos2::new(center.x, center.y + (y_offset + 48.0) * scale),
            Align2::CENTER_CENTER,
            label,
            FontId::new(22.0 * scale, FontFamily::Name("Impact".into())),
            if m.status == MatchStatus::Completed { theme::SUCCESS() } else { theme::WARNING() }
        );
    } else {
        painter.text(
            Pos2::new(center.x, center.y + y_offset * scale),
            Align2::CENTER_CENTER,
            "VS",
            FontId::new(76.0 * scale, FontFamily::Name("Impact".into())),
            theme::ACCENT_BRONZE()
        );
    }
    
    // Render Timer below score
    if app.broadcast_timer_running || app.broadcast_timer_seconds > 0 {
        let mins = app.broadcast_timer_seconds / 60;
        let secs = app.broadcast_timer_seconds % 60;
        painter.text(
            Pos2::new(center.x, center.y + 58.0 * scale),
            Align2::CENTER_CENTER,
            format!("{:02}:{:02}", mins, secs),
            FontId::new(28.0 * scale, FontFamily::Name("Impact".into())),
            if app.broadcast_timer_running { theme::SUCCESS() } else { theme::TEXT_PRIMARY() }
        );
    }
}

fn render_next_match(app: &TourviaApp, ui: &mut egui::Ui, current: &Match, scale: f32) {
    if let Some(next) = next_match(app, current) {
        let text_color = theme::TEXT_PRIMARY();
        let muted = theme::TEXT_MUTED();
        let bronze = theme::ACCENT_BRONZE_LIGHT();
        
        ui.vertical_centered(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(720.0 * scale, 60.0 * scale), egui::Sense::hover());
            let painter = ui.painter();
            
            let slant = 20.0 * scale;
            let points = vec![
                Pos2::new(rect.left() + slant, rect.top()),
                Pos2::new(rect.right(), rect.top()),
                Pos2::new(rect.right() - slant, rect.bottom()),
                Pos2::new(rect.left(), rect.bottom()),
            ];
            
            painter.add(Shape::convex_polygon(points, Color32::from_rgb(22, 26, 30), Stroke::new(1.0, theme::BORDER_SUBTLE())));
            
            painter.text(
                Pos2::new(rect.left() + 50.0 * scale, rect.center().y),
                Align2::LEFT_CENTER,
                "NEXT UP",
                FontId::new(20.0 * scale, FontFamily::Name("Impact".into())),
                bronze
            );
            
            painter.text(
                Pos2::new(rect.left() + 160.0 * scale, rect.center().y),
                Align2::LEFT_CENTER,
                round_name(app, &next.round_id).to_uppercase(),
                FontId::new(18.0 * scale, FontFamily::Name("Impact".into())),
                muted
            );
            
            painter.text(
                Pos2::new(rect.right() - 50.0 * scale, rect.center().y),
                Align2::RIGHT_CENTER,
                format!("{}  VS  {}", player_name(&next.player1_name).to_uppercase(), player_name(&next.player2_name).to_uppercase()),
                FontId::new(22.0 * scale, FontFamily::Name("Impact".into())),
                text_color
            );
        });
    }
}

fn draw_logo_placeholder(painter: &egui::Painter, center: Pos2, size: f32, scale: f32) {
    let rect = Rect::from_center_size(center, Vec2::new(size, size));
    painter.rect_filled(rect, 16.0 * scale, Color32::from_rgb(30, 35, 40));
    painter.rect_stroke(rect, 16.0 * scale, Stroke::new(2.0 * scale, theme::BORDER_SUBTLE()), egui::StrokeKind::Inside);
    painter.text(
        center,
        Align2::CENTER_CENTER,
        "TBD",
        FontId::new(36.0 * scale, FontFamily::Name("Impact".into())),
        theme::TEXT_MUTED()
    );
}

fn render_empty(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(250.0);
        ui.label(
            RichText::new("NO MATCH AVAILABLE")
                .font(FontId::new(48.0, FontFamily::Name("Impact".into())))
                .color(theme::TEXT_PRIMARY())
        );
        ui.add_space(16.0);
        ui.label(
            RichText::new("GENERATE A BRACKET BEFORE OPENING THE DISPLAY.")
                .font(FontId::new(20.0, FontFamily::Name("Impact".into())))
                .color(theme::TEXT_MUTED())
        );
    });
}

fn ensure_scoreboard_selection(app: &mut TourviaApp) {
    let matches = selectable_matches(app);
    if matches.is_empty() {
        app.selected_match = None;
        return;
    }

    let selected_exists = app
        .selected_match
        .as_ref()
        .map(|id| matches.iter().any(|m| &m.id == id))
        .unwrap_or(false);

    if selected_exists {
        return;
    }

    let fallback_id = matches
        .iter()
        .find(|m| m.status == MatchStatus::InProgress)
        .or_else(|| {
            matches.iter().find(|m| {
                m.status == MatchStatus::Pending
                    && !m.player1_name.is_empty()
                    && !m.player2_name.is_empty()
            })
        })
        .or_else(|| matches.iter().find(|m| m.status == MatchStatus::Pending))
        .or_else(|| {
            matches
                .iter()
                .rev()
                .find(|m| m.status == MatchStatus::Completed)
        })
        .or_else(|| matches.first())
        .map(|m| m.id.clone());

    if app.selected_match != fallback_id {
        app.score_input = [String::new(), String::new()];
    }
    app.selected_match = fallback_id;
}

fn current_match(app: &TourviaApp) -> Option<Match> {
    let selected = app.selected_match.as_ref()?;
    app.matches.iter().find(|m| &m.id == selected).cloned()
}

fn selectable_matches(app: &TourviaApp) -> Vec<Match> {
    app.matches
        .iter()
        .filter(|m| m.status != MatchStatus::Bye)
        .cloned()
        .collect()
}

fn next_match(app: &TourviaApp, current: &Match) -> Option<Match> {
    let matches = selectable_matches(app);
    let current_idx = matches.iter().position(|m| m.id == current.id)?;

    matches
        .iter()
        .skip(current_idx + 1)
        .find(|m| m.status == MatchStatus::InProgress || m.status == MatchStatus::Pending)
        .cloned()
        .or_else(|| {
            matches
                .iter()
                .take(current_idx)
                .find(|m| m.status == MatchStatus::InProgress || m.status == MatchStatus::Pending)
                .cloned()
        })
}

fn round_name(app: &TourviaApp, round_id: &str) -> String {
    app.rounds
        .iter()
        .find(|r| r.id == round_id)
        .map(|r| r.name.clone())
        .unwrap_or_else(|| "Match".to_string())
}

fn player_name(name: &str) -> &str {
    if name.trim().is_empty() {
        "TBD"
    } else {
        name
    }
}

fn status_info(status: &MatchStatus) -> (&'static str, egui::Color32) {
    match status {
        MatchStatus::Pending => ("Pending", theme::TEXT_MUTED()),
        MatchStatus::InProgress => ("Live Match", theme::ACCENT_BRONZE()),
        MatchStatus::Completed => ("Final Result", theme::SUCCESS()),
        MatchStatus::Bye => ("Bye", theme::WARNING()),
    }
}
