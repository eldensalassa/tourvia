use std::collections::HashMap;
use image::{Rgba, RgbaImage, ImageBuffer, GenericImageView};
use imageproc::drawing::{draw_text_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use ab_glyph::{FontRef, PxScale};
use crate::domain::tournament::{Tournament, TournamentType};
use crate::domain::round::Round;
use crate::domain::match_model::{Match, MatchStatus, BracketType};

// ─── Layout Constants (identical to bracket_view.rs at zoom 1.0) ────────────
const CARD_W: i32 = 200;    // MATCH_CARD_WIDTH
const CARD_H: i32 = 60;     // MATCH_CARD_HEIGHT
const H_GAP: i32 = 60;      // ROUND_HORIZONTAL_GAP
const V_GAP: i32 = 30;      // MATCH_VERTICAL_GAP
const HEADER_H: i32 = 40;   // ROUND_HEADER_HEIGHT
const PADDING: i32 = 60;
const HALF_H: i32 = CARD_H / 2;
const SCORE_BOX_W: i32 = 30; // score_box_w in bracket_view

// ─── Theme Colors (dark theme from theme.rs) ────────────────────────────────
const BG_DARK: Rgba<u8>       = Rgba([9, 9, 11, 255]);
const BG_PANEL: Rgba<u8>      = Rgba([24, 24, 27, 255]);
const BG_CARD: Rgba<u8>       = Rgba([39, 39, 42, 255]);
const ACCENT_BRONZE: Rgba<u8> = Rgba([197, 160, 89, 255]);
const ACCENT_LIGHT: Rgba<u8>  = Rgba([230, 201, 142, 255]);
const TEXT_PRIMARY: Rgba<u8>   = Rgba([250, 250, 250, 255]);
const TEXT_SECONDARY: Rgba<u8> = Rgba([161, 161, 170, 255]);
const TEXT_MUTED: Rgba<u8>     = Rgba([113, 113, 122, 255]);
const BORDER_SUBTLE: Rgba<u8>  = Rgba([39, 39, 42, 255]);
const CONNECTOR: Rgba<u8>     = Rgba([82, 82, 91, 255]);
const SUCCESS: Rgba<u8>       = Rgba([16, 185, 129, 255]);
const GOLD: Rgba<u8>          = Rgba([212, 175, 55, 255]);
const WARNING: Rgba<u8>       = Rgba([245, 158, 11, 255]);

/// Load a font from system fonts directory (cross-platform fallback).
fn load_font() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let font_paths = [
        // Windows
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\tahoma.ttf",
        // macOS
        "/Library/Fonts/Arial.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        // Linux (Ubuntu, Debian, Arch, Fedora)
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
        "/usr/share/fonts/gnu-free/FreeSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];

    for path in &font_paths {
        if let Ok(data) = std::fs::read(path) {
            return Ok(data);
        }
    }
    
    Err("No suitable font found on the system. Please ensure standard fonts are installed.".into())
}

/// Draw a filled rounded-corner rectangle. We approximate rounded corners
/// by drawing the main rect + overlapping rects for simplicity since imageproc
/// doesn't have a built-in rounded rect.
fn draw_card_rect(image: &mut RgbaImage, x: i32, y: i32, w: i32, h: i32, fill: Rgba<u8>, border: Rgba<u8>) {
    // Fill
    if x >= 0 && y >= 0 {
        let rect = Rect::at(x, y).of_size(w as u32, h as u32);
        draw_filled_rect_mut(image, rect, fill);
    }
    // Top border
    draw_h_line(image, x, x + w, y, border);
    // Bottom border
    draw_h_line(image, x, x + w, y + h - 1, border);
    // Left border
    draw_v_line(image, x, y, y + h, border);
    // Right border
    draw_v_line(image, x + w - 1, y, y + h, border);
}

fn draw_h_line(image: &mut RgbaImage, x1: i32, x2: i32, y: i32, color: Rgba<u8>) {
    if y < 0 || y >= image.height() as i32 { return; }
    for x in x1.max(0)..x2.min(image.width() as i32) {
        image.put_pixel(x as u32, y as u32, color);
    }
}

fn draw_v_line(image: &mut RgbaImage, x: i32, y1: i32, y2: i32, color: Rgba<u8>) {
    if x < 0 || x >= image.width() as i32 { return; }
    let (ya, yb) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
    for y in ya.max(0)..yb.min(image.height() as i32) {
        image.put_pixel(x as u32, y as u32, color);
    }
}

/// Draw a thick line (2px) for connectors.
fn draw_connector_h(image: &mut RgbaImage, x1: i32, x2: i32, y: i32, color: Rgba<u8>) {
    draw_h_line(image, x1, x2, y, color);
    draw_h_line(image, x1, x2, y + 1, color);
}

fn draw_connector_v(image: &mut RgbaImage, x: i32, y1: i32, y2: i32, color: Rgba<u8>) {
    draw_v_line(image, x, y1, y2, color);
    draw_v_line(image, x + 1, y1, y2, color);
}

/// Truncate a string to max_chars and append "…" if too long.
fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        format!("{}…", s.chars().take(max_chars - 1).collect::<String>())
    } else {
        s.to_string()
    }
}

/// Decode raw image bytes (PNG/JPEG) and overlay onto the target image at (x, y)
/// with the given size, preserving aspect ratio and alpha.
fn overlay_logo(target: &mut RgbaImage, raw_data: &[u8], x: i32, y: i32, size: i32) {
    let logo = match image::load_from_memory(raw_data) {
        Ok(img) => img,
        Err(_) => return,
    };
    let resized = logo.resize(
        size as u32, size as u32,
        image::imageops::FilterType::Lanczos3,
    );
    let rw = resized.width() as i32;
    let rh = resized.height() as i32;
    // Center within the size box
    let ox = x + (size - rw) / 2;
    let oy = y + (size - rh) / 2;
    let (tw, th) = (target.width() as i32, target.height() as i32);
    for py in 0..rh {
        for px in 0..rw {
            let tx = ox + px;
            let ty = oy + py;
            if tx >= 0 && tx < tw && ty >= 0 && ty < th {
                let src = resized.get_pixel(px as u32, py as u32);
                let alpha = src[3] as f32 / 255.0;
                if alpha > 0.0 {
                    let dst = target.get_pixel(tx as u32, ty as u32);
                    let blend = |s: u8, d: u8| -> u8 {
                        (s as f32 * alpha + d as f32 * (1.0 - alpha)).min(255.0) as u8
                    };
                    target.put_pixel(
                        tx as u32, ty as u32,
                        Rgba([blend(src[0], dst[0]), blend(src[1], dst[1]), blend(src[2], dst[2]), 255]),
                    );
                }
            }
        }
    }
}

/// Main export function. Renders the bracket to a PNG file, faithfully
/// replicating the layout from bracket_view.rs.
pub fn export_bracket(
    tournament: &Tournament,
    rounds: &[Round],
    matches: &[Match],
    output_path: &str,
    champion_name: Option<&str>,
    participant_logos: &HashMap<String, Vec<u8>>,
    tournament_logo: Option<&[u8]>,
) -> Result<(), Box<dyn std::error::Error>> {
    let font_data = load_font()?;
    let font = FontRef::try_from_slice(&font_data).map_err(|_| "Failed to load font")?;

    if rounds.is_empty() || matches.is_empty() {
        // Minimal image with "No matches" message
        let mut image = ImageBuffer::from_pixel(800, 200, BG_DARK);
        draw_text_mut(&mut image, ACCENT_LIGHT, 50, 30, PxScale::from(36.0), &font,
            &format!("{} - Bracket", tournament.name));
        draw_text_mut(&mut image, TEXT_SECONDARY, 50, 90, PxScale::from(24.0), &font,
            "No matches to display.");
        image.save(output_path)?;
        return Ok(());
    }

    if tournament.tournament_type == TournamentType::RoundRobin {
        return export_round_robin(tournament, rounds, matches, output_path, champion_name, &font, participant_logos, tournament_logo);
    }

    export_elimination(tournament, rounds, matches, output_path, champion_name, &font, participant_logos, tournament_logo)
}

// ═══════════════════════════════════════════════════════════════════════════════
// ELIMINATION BRACKET EXPORT
// ═══════════════════════════════════════════════════════════════════════════════

fn export_elimination(
    tournament: &Tournament,
    rounds: &[Round],
    matches: &[Match],
    output_path: &str,
    champion_name: Option<&str>,
    font: &FontRef,
    participant_logos: &HashMap<String, Vec<u8>>,
    tournament_logo: Option<&[u8]>,
) -> Result<(), Box<dyn std::error::Error>> {
    // ── Classify rounds by bracket type ─────────────────────────────────────
    let upper_rounds: Vec<&Round> = rounds.iter().filter(|r| r.bracket_type == BracketType::Upper).collect();
    let lower_rounds: Vec<&Round> = rounds.iter().filter(|r| r.bracket_type == BracketType::Lower).collect();
    let gf_rounds: Vec<&Round> = rounds.iter().filter(|r| r.bracket_type == BracketType::GrandFinal).collect();
    let tp_rounds: Vec<&Round> = rounds.iter().filter(|r| r.bracket_type == BracketType::ThirdPlace).collect();

    let first_round_matches_upper = if !upper_rounds.is_empty() {
        matches.iter().filter(|m| m.round_id == upper_rounds[0].id).count()
    } else { 0 };

    let first_round_matches_lower = if !lower_rounds.is_empty() {
        matches.iter().filter(|m| m.round_id == lower_rounds[0].id).count()
    } else { 0 };

    let num_upper_rounds = upper_rounds.len();
    let num_lower_rounds = lower_rounds.len();
    let max_rounds = num_upper_rounds.max(num_lower_rounds);

    // ── Compute section heights ─────────────────────────────────────────────
    let upper_height = if first_round_matches_upper > 0 {
        first_round_matches_upper as i32 * (CARD_H + V_GAP) + HEADER_H + 40
    } else {
        HEADER_H + CARD_H + V_GAP + 40
    };

    let lower_height = if first_round_matches_lower > 0 {
        first_round_matches_lower as i32 * (CARD_H + V_GAP) + HEADER_H + 40
    } else { 0 };

    // Title area
    let title_area = 70;
    let champion_area = if champion_name.is_some() { 60 } else { 0 };
    let top_offset = PADDING + title_area + champion_area;

    // Total height
    let mut total_content_height = upper_height;
    if num_lower_rounds > 0 {
        total_content_height += lower_height + 50;
    }
    if !tp_rounds.is_empty() {
        // Calculate the bottom Y of the 3rd place bracket to ensure it's fully visible
        let tree_origin_y = 40;
        let first_round_total_height = first_round_matches_upper as i32 * (CARD_H + V_GAP);
        let upper_final_y = tree_origin_y + HEADER_H + (first_round_total_height - CARD_H) / 2;
        let target_y = upper_final_y + CARD_H + HEADER_H + 20;
        let tp_y_rel = target_y - 15 - HEADER_H - V_GAP / 2;
        let tp_bottom = tp_y_rel + CARD_H + HEADER_H + 40;
        total_content_height = total_content_height.max(tp_bottom);
    }

    let canvas_height = (top_offset + total_content_height + PADDING) as u32;

    // Total width
    let total_columns = max_rounds + gf_rounds.len();
    let canvas_width = (PADDING * 2 + total_columns as i32 * (CARD_W + H_GAP) + H_GAP) as u32;

    let canvas_width = canvas_width.max(900);
    let canvas_height = canvas_height.max(500);

    let mut image = ImageBuffer::from_pixel(canvas_width, canvas_height, BG_DARK);

    // ── Draw Title with Tournament Logo ──────────────────────────────────────
    let mut title_text_x = PADDING;
    if let Some(logo_bytes) = tournament_logo {
        let logo_sz = 48;
        overlay_logo(&mut image, logo_bytes, PADDING, PADDING - 4, logo_sz);
        title_text_x += logo_sz + 12;
    }
    let title = format!("{} - Bracket", tournament.name);
    draw_text_mut(&mut image, ACCENT_LIGHT, title_text_x, PADDING, PxScale::from(32.0), font, &title);

    let type_label = format!("Format: {}", tournament.tournament_type.as_str());
    draw_text_mut(&mut image, TEXT_MUTED, title_text_x, PADDING + 40, PxScale::from(16.0), font, &type_label);

    // ── Draw Champion Banner ────────────────────────────────────────────────
    if let Some(champ) = champion_name {
        let banner_y = PADDING + title_area;
        let banner_rect = Rect::at(PADDING, banner_y).of_size((canvas_width as i32 - PADDING * 2) as u32, 44);
        // Semi-transparent gold tint background
        draw_filled_rect_mut(&mut image, banner_rect, Rgba([50, 40, 15, 255]));
        // Border
        draw_h_line(&mut image, PADDING, canvas_width as i32 - PADDING, banner_y, ACCENT_BRONZE);
        draw_h_line(&mut image, PADDING, canvas_width as i32 - PADDING, banner_y + 43, ACCENT_BRONZE);
        // Text
        let champ_text = format!("Champion: {}", champ);
        draw_text_mut(&mut image, GOLD, PADDING + 16, banner_y + 10, PxScale::from(22.0), font, &champ_text);
    }

    let origin_x = PADDING;
    let origin_y = top_offset;

    // ── Draw Upper Bracket ──────────────────────────────────────────────────
    if !upper_rounds.is_empty() {
        let label = if num_lower_rounds > 0 { "Upper Bracket" } else { "" };
        draw_bracket_tree(&mut image, font, &upper_rounds, matches, origin_x, origin_y, label, participant_logos);
    }

    // ── Draw Lower Bracket ──────────────────────────────────────────────────
    let lower_origin_y = origin_y + upper_height + 20;
    if !lower_rounds.is_empty() {
        draw_bracket_tree(&mut image, font, &lower_rounds, matches, origin_x, lower_origin_y, "Lower Bracket", participant_logos);
    }

    // ── Draw Grand Final ────────────────────────────────────────────────────
    if !gf_rounds.is_empty() {
        let gf_x = origin_x + max_rounds as i32 * (CARD_W + H_GAP);
        let gf_y = origin_y + (total_content_height - CARD_H - HEADER_H) / 2 - 40;
        draw_bracket_tree(&mut image, font, &gf_rounds, matches, gf_x, gf_y, "", participant_logos);

        // Draw connections from Upper Final and Lower Final to GF
        if let Some(_gf_match) = matches.iter().find(|m| m.round_id == gf_rounds[0].id) {
            let gf_card_x = gf_x + H_GAP / 2;
            let gf_first_count = matches.iter().filter(|m| m.round_id == gf_rounds[0].id).count();
            let gf_first_total = gf_first_count as i32 * (CARD_H + V_GAP);
            let gf_card_y = gf_y + 40 + HEADER_H + (gf_first_total - CARD_H) / 2;
            let mid_x = gf_card_x - H_GAP / 2;
            let gf_center_y = gf_card_y + HALF_H;

            // Upper Final connection
            if !upper_rounds.is_empty() {
                let ur_idx = upper_rounds.len() - 1;
                let uf_x = origin_x + H_GAP / 2 + ur_idx as i32 * (CARD_W + H_GAP) + CARD_W;
                let upper_first_count = matches.iter().filter(|m| m.round_id == upper_rounds[0].id).count();
                let upper_first_total = upper_first_count as i32 * (CARD_H + V_GAP);
                let uf_y = origin_y + 40 + HEADER_H + (upper_first_total - CARD_H) / 2 + HALF_H;

                draw_connector_h(&mut image, uf_x, mid_x, uf_y, CONNECTOR);
                draw_connector_v(&mut image, mid_x, uf_y.min(gf_center_y), uf_y.max(gf_center_y), CONNECTOR);
            }

            // Lower Final connection
            if !lower_rounds.is_empty() {
                let lr_idx = lower_rounds.len() - 1;
                let lf_x = origin_x + H_GAP / 2 + lr_idx as i32 * (CARD_W + H_GAP) + CARD_W;
                let lower_first_count = matches.iter().filter(|m| m.round_id == lower_rounds[0].id).count();
                let lower_first_total = lower_first_count as i32 * (CARD_H + V_GAP);
                let lf_y = lower_origin_y + 40 + HEADER_H + (lower_first_total - CARD_H) / 2 + HALF_H;

                draw_connector_h(&mut image, lf_x, mid_x, lf_y, CONNECTOR);
                draw_connector_v(&mut image, mid_x, lf_y.min(gf_center_y), lf_y.max(gf_center_y), CONNECTOR);
            }

            // Horizontal into GF card
            draw_connector_h(&mut image, mid_x, gf_card_x, gf_center_y, CONNECTOR);
        }
    }

    // ── Draw 3rd Place ──────────────────────────────────────────────────────
    if !tp_rounds.is_empty() {
        let tp_x = origin_x + (max_rounds as i32 - 1).max(0) * (CARD_W + H_GAP);
        let mut tp_y = origin_y + upper_height + 1;
        if !upper_rounds.is_empty() {
            let tree_origin_y = origin_y + 40;
            let first_round_total_height = first_round_matches_upper as i32 * (CARD_H + V_GAP);
            let upper_final_y = tree_origin_y + HEADER_H + (first_round_total_height - CARD_H) / 2;
            let target_y = upper_final_y + CARD_H + HEADER_H + 20;
            tp_y = target_y - 15 - HEADER_H - V_GAP / 2;
        }
        draw_bracket_tree(&mut image, font, &tp_rounds, matches, tp_x, tp_y, "", participant_logos);
    }

    // ── Watermark ───────────────────────────────────────────────────────────
    draw_text_mut(&mut image, TEXT_MUTED, canvas_width as i32 - 200, canvas_height as i32 - 30,
        PxScale::from(12.0), font, "Generated by Tourvia");

    image.save(output_path)?;
    Ok(())
}

/// Draw a bracket tree section (upper, lower, GF, etc.) exactly mirroring
/// the `draw_tree` closure in bracket_view.rs.
fn draw_bracket_tree(
    image: &mut RgbaImage,
    font: &FontRef,
    rounds: &[&Round],
    matches: &[Match],
    start_x: i32,
    start_y: i32,
    label: &str,
    participant_logos: &HashMap<String, Vec<u8>>,
) {
    if rounds.is_empty() { return; }

    // Section label
    if !label.is_empty() {
        draw_text_mut(image, TEXT_PRIMARY, start_x + 20, start_y + 4, PxScale::from(18.0), font, label);
    }

    let tree_y = start_y + 40;

    let first_round_count = matches.iter().filter(|m| m.round_id == rounds[0].id).count();
    let first_round_total_height = first_round_count as i32 * (CARD_H + V_GAP);

    for (round_idx, round) in rounds.iter().enumerate() {
        let round_x = start_x + H_GAP / 2 + round_idx as i32 * (CARD_W + H_GAP);

        // Round header (centered above the column, matching Align2::CENTER_CENTER)
        let header_text_x = round_x + 10; // approximate left-aligned within column
        draw_text_mut(image, TEXT_SECONDARY, header_text_x, tree_y + 12,
            PxScale::from(14.0), font, &round.name);

        let round_matches: Vec<&Match> = matches.iter()
            .filter(|m| m.round_id == round.id)
            .collect();
        let matches_in_round = round_matches.len();
        let vertical_spacing = if matches_in_round > 0 {
            first_round_total_height / matches_in_round as i32
        } else { 0 };

        for (match_idx, m) in round_matches.iter().enumerate() {
            let match_y = tree_y + HEADER_H + match_idx as i32 * vertical_spacing
                + (vertical_spacing - CARD_H) / 2;

            // ── Draw match card ─────────────────────────────────────────
            draw_card_rect(image, round_x, match_y, CARD_W, CARD_H, BG_CARD, BORDER_SUBTLE);

            // Divider line
            let div_y = match_y + HALF_H;
            draw_h_line(image, round_x, round_x + CARD_W, div_y, BORDER_SUBTLE);

            // Player 1 — vertically centered in top half (Align2::LEFT_CENTER)
            let p1 = if m.player1_name.is_empty() { "TBD" } else { &m.player1_name };
            let p1_win = m.winner_id.is_some() && m.player1_id == m.winner_id;
            let p1_color = if p1_win { ACCENT_BRONZE } else if m.player1_name == "BYE" { TEXT_MUTED } else { TEXT_SECONDARY };
            let p1_display = truncate(p1, 14);

            // Logo + text for Player 1 (matching bracket_view.rs: logo_size = 14)
            let logo_size: i32 = 14;
            let mut p1_text_x = round_x + 8;
            if let Some(ref id) = m.player1_id {
                if let Some(logo_data) = participant_logos.get(id) {
                    let logo_y = match_y + HALF_H / 2 - logo_size / 2;
                    overlay_logo(image, logo_data, p1_text_x, logo_y, logo_size);
                    p1_text_x += logo_size + 6;
                }
            }
            draw_text_mut(image, p1_color, p1_text_x, match_y + 9, PxScale::from(12.0), font, &p1_display);

            // Player 2 — vertically centered in bottom half
            let p2 = if m.player2_name.is_empty() { "TBD" } else { &m.player2_name };
            let p2_win = m.winner_id.is_some() && m.player2_id == m.winner_id;
            let p2_color = if p2_win { ACCENT_BRONZE } else if m.player2_name == "BYE" { TEXT_MUTED } else { TEXT_SECONDARY };
            let p2_display = truncate(p2, 14);

            // Logo + text for Player 2
            let mut p2_text_x = round_x + 8;
            if let Some(ref id) = m.player2_id {
                if let Some(logo_data) = participant_logos.get(id) {
                    let logo_y = div_y + HALF_H / 2 - logo_size / 2;
                    overlay_logo(image, logo_data, p2_text_x, logo_y, logo_size);
                    p2_text_x += logo_size + 6;
                }
            }
            draw_text_mut(image, p2_color, p2_text_x, div_y + 9, PxScale::from(12.0), font, &p2_display);

            // Scores (only if completed or in progress)
            if m.status == MatchStatus::Completed || m.status == MatchStatus::InProgress {
                // Score box backgrounds
                let score1_x = round_x + CARD_W - SCORE_BOX_W;
                let score1_rect = Rect::at(score1_x, match_y).of_size(SCORE_BOX_W as u32, HALF_H as u32);
                draw_filled_rect_mut(image, score1_rect, BG_PANEL);
                draw_v_line(image, score1_x, match_y, match_y + HALF_H, BORDER_SUBTLE);

                let score2_rect = Rect::at(score1_x, div_y).of_size(SCORE_BOX_W as u32, HALF_H as u32);
                draw_filled_rect_mut(image, score2_rect, BG_PANEL);
                draw_v_line(image, score1_x, div_y, div_y + HALF_H, BORDER_SUBTLE);

                // Score text
                let s1_color = if p1_win { SUCCESS } else { TEXT_MUTED };
                let s2_color = if p2_win { SUCCESS } else { TEXT_MUTED };

                // Centered in score box (Align2::CENTER_CENTER)
                draw_text_mut(image, s1_color, score1_x + 8, match_y + 9, PxScale::from(12.0), font, &m.score1.to_string());
                draw_text_mut(image, s2_color, score1_x + 8, div_y + 9, PxScale::from(12.0), font, &m.score2.to_string());

                // Winner indicator (green bar on right edge)
                if p1_win {
                    draw_v_line(image, round_x + CARD_W - 2, match_y + 2, match_y + HALF_H - 2, SUCCESS);
                    draw_v_line(image, round_x + CARD_W - 3, match_y + 2, match_y + HALF_H - 2, SUCCESS);
                }
                if p2_win {
                    draw_v_line(image, round_x + CARD_W - 2, div_y + 2, div_y + HALF_H - 2, SUCCESS);
                    draw_v_line(image, round_x + CARD_W - 3, div_y + 2, div_y + HALF_H - 2, SUCCESS);
                }
            }

            // BYE label
            if m.status == MatchStatus::Bye {
                draw_text_mut(image, TEXT_MUTED, round_x + CARD_W - 40, match_y + HALF_H - 8,
                    PxScale::from(11.0), font, "BYE");
            }

            // ── Connector lines to next match ───────────────────────────
            if round_idx < rounds.len() - 1 {
                if let Some(ref next_match_id) = m.next_match_id {
                    let next_round = rounds[round_idx + 1];
                    let next_round_matches: Vec<&Match> = matches.iter()
                        .filter(|nm| nm.round_id == next_round.id)
                        .collect();

                    if let Some((next_idx, _)) = next_round_matches.iter().enumerate()
                        .find(|(_, nm)| nm.id == *next_match_id)
                    {
                        let next_vs = if !next_round_matches.is_empty() {
                            first_round_total_height / next_round_matches.len() as i32
                        } else { 0 };

                        let next_x = start_x + H_GAP / 2 + (round_idx as i32 + 1) * (CARD_W + H_GAP);
                        let next_y = tree_y + HEADER_H + next_idx as i32 * next_vs + (next_vs - CARD_H) / 2;

                        let start_cx = round_x + CARD_W;
                        let start_cy = match_y + HALF_H;
                        let end_cx = next_x;
                        let end_cy = next_y + HALF_H;
                        let mid_x = start_cx + H_GAP / 2;

                        // L-shaped connector: horizontal → vertical → horizontal
                        draw_connector_h(image, start_cx, mid_x, start_cy, CONNECTOR);
                        draw_connector_v(image, mid_x, start_cy.min(end_cy), start_cy.max(end_cy), CONNECTOR);
                        draw_connector_h(image, mid_x, end_cx, end_cy, CONNECTOR);
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ROUND ROBIN EXPORT
// ═══════════════════════════════════════════════════════════════════════════════

fn export_round_robin(
    tournament: &Tournament,
    rounds: &[Round],
    matches: &[Match],
    output_path: &str,
    champion_name: Option<&str>,
    font: &FontRef,
    participant_logos: &HashMap<String, Vec<u8>>,
    tournament_logo: Option<&[u8]>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rr_card_w: i32 = 260;
    let rr_card_h: i32 = 70;
    let cards_per_row: i32 = 3;
    let card_spacing: i32 = 16;

    let title_area = 70;
    let champion_area = if champion_name.is_some() { 60 } else { 0 };
    let top_offset = PADDING + title_area + champion_area;

    // Calculate total height
    let mut total_height = top_offset;
    for round in rounds {
        let round_matches_count = matches.iter().filter(|m| m.round_id == round.id).count() as i32;
        let rows = (round_matches_count + cards_per_row - 1) / cards_per_row;
        total_height += 50 + rows * (rr_card_h + card_spacing) + 20;
    }
    total_height += PADDING;

    let canvas_width = (PADDING * 2 + cards_per_row * (rr_card_w + card_spacing)) as u32;
    let canvas_height = total_height.max(400) as u32;

    let mut image = ImageBuffer::from_pixel(canvas_width, canvas_height, BG_DARK);

    // Title with Tournament Logo
    let mut title_text_x = PADDING;
    if let Some(logo_bytes) = tournament_logo {
        let logo_sz = 48;
        overlay_logo(&mut image, logo_bytes, PADDING, PADDING - 4, logo_sz);
        title_text_x += logo_sz + 12;
    }
    let title = format!("{} - Round Robin", tournament.name);
    draw_text_mut(&mut image, ACCENT_LIGHT, title_text_x, PADDING, PxScale::from(32.0), font, &title);

    // Champion banner
    if let Some(champ) = champion_name {
        let banner_y = PADDING + title_area;
        let banner_rect = Rect::at(PADDING, banner_y).of_size((canvas_width as i32 - PADDING * 2) as u32, 44);
        draw_filled_rect_mut(&mut image, banner_rect, Rgba([50, 40, 15, 255]));
        draw_h_line(&mut image, PADDING, canvas_width as i32 - PADDING, banner_y, ACCENT_BRONZE);
        draw_h_line(&mut image, PADDING, canvas_width as i32 - PADDING, banner_y + 43, ACCENT_BRONZE);
        let champ_text = format!("Champion: {}", champ);
        draw_text_mut(&mut image, GOLD, PADDING + 16, banner_y + 10, PxScale::from(22.0), font, &champ_text);
    }

    let mut y_cursor = top_offset;

    for round in rounds {
        let round_matches: Vec<&Match> = matches.iter()
            .filter(|m| m.round_id == round.id)
            .collect();

        // Round header background
        let header_rect = Rect::at(PADDING, y_cursor).of_size((canvas_width as i32 - PADDING * 2) as u32, 36);
        draw_filled_rect_mut(&mut image, header_rect, BG_PANEL);
        draw_text_mut(&mut image, ACCENT_LIGHT, PADDING + 12, y_cursor + 8, PxScale::from(16.0), font, &round.name);

        // Completion badge
        let total = round_matches.len();
        let completed = round_matches.iter().filter(|m| m.status == MatchStatus::Completed).count();
        let badge_text = if total > 0 && completed == total {
            "Round Complete".to_string()
        } else {
            format!("{}/{} Completed", completed, total)
        };
        let badge_color = if total > 0 && completed == total { SUCCESS } else { TEXT_MUTED };
        draw_text_mut(&mut image, badge_color,
            canvas_width as i32 - PADDING - 160, y_cursor + 10,
            PxScale::from(12.0), font, &badge_text);

        y_cursor += 44;

        // Match cards in grid
        for (i, m) in round_matches.iter().enumerate() {
            let col = i as i32 % cards_per_row;
            let row = i as i32 / cards_per_row;
            let cx = PADDING + col * (rr_card_w + card_spacing);
            let cy = y_cursor + row * (rr_card_h + card_spacing);

            draw_card_rect(&mut image, cx, cy, rr_card_w, rr_card_h, BG_CARD, BORDER_SUBTLE);

            let p1 = if m.player1_name.is_empty() { "TBD" } else { &m.player1_name };
            let p2 = if m.player2_name.is_empty() { "TBD" } else { &m.player2_name };
            let p1_win = m.winner_id.is_some() && m.player1_id == m.winner_id;
            let p2_win = m.winner_id.is_some() && m.player2_id == m.winner_id;

            let p1_color = if p1_win { SUCCESS } else { TEXT_PRIMARY };
            let p2_color = if p2_win { SUCCESS } else { TEXT_PRIMARY };

            // Player 1 with logo
            let rr_logo_size: i32 = 16;
            let mut p1_tx = cx + 10;
            if let Some(ref id) = m.player1_id {
                if let Some(logo_data) = participant_logos.get(id) {
                    overlay_logo(&mut image, logo_data, p1_tx, cy + 4, rr_logo_size);
                    p1_tx += rr_logo_size + 4;
                }
            }
            draw_text_mut(&mut image, p1_color, p1_tx, cy + 8, PxScale::from(12.0), font, &truncate(p1, 12));

            // Player 2 with logo
            let mut p2_tx = cx + rr_card_w - 90;
            if let Some(ref id) = m.player2_id {
                if let Some(logo_data) = participant_logos.get(id) {
                    overlay_logo(&mut image, logo_data, p2_tx, cy + 4, rr_logo_size);
                    p2_tx += rr_logo_size + 4;
                }
            }
            draw_text_mut(&mut image, p2_color, p2_tx, cy + 8, PxScale::from(12.0), font, &truncate(p2, 12));

            // VS / Score
            if m.status == MatchStatus::Completed {
                let score_text = format!("{} - {}", m.score1, m.score2);
                draw_text_mut(&mut image, TEXT_PRIMARY, cx + rr_card_w / 2 - 15, cy + 28, PxScale::from(16.0), font, &score_text);
            } else {
                draw_text_mut(&mut image, TEXT_MUTED, cx + rr_card_w / 2 - 8, cy + 28, PxScale::from(13.0), font, "VS");
            }

            // Status badge
            let (badge_bg, badge_fg, badge_label) = match m.status {
                MatchStatus::Completed => (Rgba([10, 40, 25, 255]), SUCCESS, "FT"),
                MatchStatus::InProgress => (Rgba([50, 40, 15, 255]), ACCENT_BRONZE, "LIVE"),
                MatchStatus::Bye => (Rgba([50, 50, 15, 255]), WARNING, "BYE"),
                MatchStatus::Pending => (BG_PANEL, TEXT_MUTED, "PENDING"),
            };
            let badge_x = cx + rr_card_w / 2 - 20;
            let badge_y = cy + 48;
            let badge_rect = Rect::at(badge_x, badge_y).of_size(42, 16);
            draw_filled_rect_mut(&mut image, badge_rect, badge_bg);
            draw_text_mut(&mut image, badge_fg, badge_x + 4, badge_y + 2, PxScale::from(10.0), font, badge_label);
        }

        let round_matches_count = round_matches.len() as i32;
        let rows = (round_matches_count + cards_per_row - 1) / cards_per_row;
        y_cursor += rows * (rr_card_h + card_spacing) + 20;
    }

    // Watermark
    draw_text_mut(&mut image, TEXT_MUTED, canvas_width as i32 - 200, canvas_height as i32 - 30,
        PxScale::from(12.0), font, "Generated by Tourvia");

    image.save(output_path)?;
    Ok(())
}
