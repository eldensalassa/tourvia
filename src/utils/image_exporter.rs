use image::{RgbaImage, Rgba, ImageBuffer};
use imageproc::drawing::{draw_line_segment_mut, draw_text_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use ab_glyph::{FontRef, PxScale};
use crate::domain::tournament::{Tournament, TournamentType};
use crate::domain::round::Round;
use crate::domain::match_model::Match;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const BG_COLOR: Rgba<u8> = Rgba([18, 18, 18, 255]);
const PANEL_COLOR: Rgba<u8> = Rgba([26, 26, 26, 255]);
const LINE_COLOR: Rgba<u8> = Rgba([80, 80, 80, 255]);
const TEXT_COLOR: Rgba<u8> = Rgba([235, 235, 235, 255]);
const WINNER_COLOR: Rgba<u8> = Rgba([205, 127, 50, 255]); // Bronze
const TITLE_COLOR: Rgba<u8> = Rgba([224, 155, 89, 255]); // Light Bronze

pub fn export_bracket(
    tournament: &Tournament,
    rounds: &[Round],
    matches: &[Match],
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut image = ImageBuffer::from_pixel(WIDTH, HEIGHT, BG_COLOR);
    
    // Load default font. In a real app we'd load a TTF file from assets.
    // For simplicity without external assets, we use an embedded font if possible,
    // but rusttype requires a byte slice. Since we don't have one, we can't draw text without a font!
    // Since rusttype needs a font byte slice, let's just create a generic function.
    // Wait! We can include a system font on Windows.
    let font_data = std::fs::read("C:\\Windows\\Fonts\\arial.ttf")
        .or_else(|_| std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf"))?;
    let font = FontRef::try_from_slice(&font_data).map_err(|_| "Error constructing Font")?;

    // Draw Title
    let title = format!("{} - Bracket", tournament.name);
    draw_text_mut(&mut image, TITLE_COLOR, 50, 30, PxScale::from(40.0), &font, &title);
    
    if tournament.tournament_type != TournamentType::SingleElimination {
        draw_text_mut(&mut image, TEXT_COLOR, 50, 80, PxScale::from(20.0), &font, "Notice: Only Single Elimination view is fully supported in image export for now.");
        // Fallback for simplicity: we still render it as columns if we can
    }

    if rounds.is_empty() {
        draw_text_mut(&mut image, TEXT_COLOR, 50, 150, PxScale::from(30.0), &font, "No matches to display.");
        image.save(output_path)?;
        return Ok(());
    }

    // Bracket Layout Variables
    let match_width = 220;
    let match_height = 60;
    let col_spacing = 300;
    let start_x = 50;
    let mut current_y = 150;
    
    // We will do a simple column-based rendering instead of a complex tree 
    // if the tree logic is too hard, but wait, we can do exactly what the UI does!
    // UI draws matches grouped by round.
    
    // Basic rendering
    for (i, round) in rounds.iter().enumerate() {
        let x = start_x + (i as i32 * col_spacing);
        
        let round_matches: Vec<&Match> = matches.iter()
            .filter(|m| m.round_id == round.id)
            .collect();
            
        let mut y_offset = current_y + (i as i32 * 40); // indent later rounds
        
        draw_text_mut(&mut image, TITLE_COLOR, x, y_offset - 30, PxScale::from(20.0), &font, &format!("Round {}", round.round_number));

        for m in round_matches {
            let rect = Rect::at(x, y_offset).of_size(match_width as u32, match_height as u32);
            draw_filled_rect_mut(&mut image, rect, PANEL_COLOR);
            
            // Participants
            let p1 = if m.player1_name.is_empty() { "TBD".to_string() } else { m.player1_name.clone() };
            let p2 = if m.player2_name.is_empty() { "TBD".to_string() } else { m.player2_name.clone() };
            let s1 = m.score1.to_string();
            let s2 = m.score2.to_string();

            let c1 = if m.player1_id.is_some() && m.player1_id == m.winner_id { WINNER_COLOR } else { TEXT_COLOR };
            let c2 = if m.player2_id.is_some() && m.player2_id == m.winner_id { WINNER_COLOR } else { TEXT_COLOR };

            draw_text_mut(&mut image, c1, x + 10, y_offset + 10, PxScale::from(16.0), &font, &p1);
            draw_text_mut(&mut image, c1, x + match_width - 30, y_offset + 10, PxScale::from(16.0), &font, &s1);

            // separator
            draw_line_segment_mut(&mut image, (x as f32, (y_offset + 30) as f32), ((x + match_width) as f32, (y_offset + 30) as f32), LINE_COLOR);

            draw_text_mut(&mut image, c2, x + 10, y_offset + 35, PxScale::from(16.0), &font, &p2);
            draw_text_mut(&mut image, c2, x + match_width - 30, y_offset + 35, PxScale::from(16.0), &font, &s2);
            
            y_offset += match_height + 20; // space between matches
        }
    }

    image.save(output_path)?;
    Ok(())
}
